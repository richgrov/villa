use tokio::{net::{TcpStream, tcp::OwnedReadHalf}, io::{BufReader, AsyncReadExt, AsyncWriteExt}, sync::mpsc};

use std::io::{Error, ErrorKind};

use super::packets::{self, InboundPacket, OutboundPacket, PacketVisitor, Packet};

pub struct Connection {
    inbound_packets_rx: mpsc::Receiver<Result<Box<dyn PacketVisitor + Send>, std::io::Error>>,
    outbound_packets_tx: mpsc::Sender<Vec<u8>>,
}

impl Connection {
    pub async fn connect(address: &str, username: &str) -> Result<Connection, Error> {
        let stream = TcpStream::connect(address).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::with_capacity(1024, reader);

        writer.write_all(&packets::Handshake {
            username: username.to_owned(),
        }.serialize()).await?;

        let response_handshake: packets::Handshake = expect_packet(&mut reader).await?;
        if response_handshake.username != "-" {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("expected to authentication string from server but got {}", response_handshake.username),
            ))
        }

        writer.write_all(&packets::Login {
            protocol_version: packets::PROTOCOL_VERSION,
            username: username.to_owned(),
            seed: 0,
            dimension: 0,
        }.serialize()).await?;

        let (in_tx, in_rx) = mpsc::channel(24);
        let in_tx2 = in_tx.clone(); // used for writer task
        let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(24);

        tokio::spawn(async move {
            loop {
                let packet = Self::read_next_packet(&mut reader).await;
                let err = packet.is_err();
                in_tx.send(packet).await?;

                if err {
                    break
                }
            }

            Ok::<_, mpsc::error::SendError<_>>(())
        });

        tokio::spawn(async move {
            loop {
                match out_rx.try_recv() {
                    Ok(p) => {
                        if let Err(e) = writer.write_all(&p).await {
                            // Ignored for same reason as reader task
                            in_tx2.send(Err(e)).await?;
                        }
                    },
                    Err(mpsc::error::TryRecvError::Empty) => {},
                    Err(mpsc::error::TryRecvError::Disconnected) => break,
                };
            }

            Ok::<_, mpsc::error::SendError<_>>(())
        });

        Ok(Connection {
            inbound_packets_rx: in_rx,
            outbound_packets_tx: out_tx,
        })
    }

    async fn read_next_packet(reader: &mut BufReader<OwnedReadHalf>) -> Result<Box<dyn PacketVisitor + Send>, Error> {
        let id = reader.read_u8().await?;

        macro_rules! match_packets {
            ($($name:ident),* $(,)?) => {
                match id {
                    $(
                        packets::$name::ID => {
                            let packet = match packets::$name::deserialize(reader).await {
                                Ok(p) => p,
                                Err(e) => return Err(Error::new(
                                    ErrorKind::InvalidInput,
                                    format!("error deserializing packet id {}: {}", packets::$name::ID, e),
                                )),
                            };
                            Box::new(packet)
                        },
                    )*
                    other => return Err(Error::new(ErrorKind::InvalidInput, format!("unhandled packet id {}", other))),
                }    
            };
        }
        Ok(match_packets!(
            Login,
            Chat,
            SetTime,
            SetEntityItem,
            SetHealth,
            SpawnPos,
            Position,
            PosRot,
            SpawnPlayer,
            SpawnItemEntity,
            SpawnInsentientEntity,
            SpawnEntity,
            EntityVelocity,
            RemoveEntity,
            MoveEntity,
            EntityMoveRot,
            EntityPosRot,
            SetEntityHealth,
            UpdateEntityAttributes,
            InitChunk,
            SetContiguousBlocks,
            SetBlocks,
            SetBlock,
            AfterRespawn,
            SetInventorySlot,
            SetInventoryItems,
            Statistic,
            Disconnect,
        ))
    }

    pub fn queue_packet<P: OutboundPacket>(&self, packet: &P) -> bool {
        match self.outbound_packets_tx.try_send(packet.serialize()) {
            Err(mpsc::error::TrySendError::Full(_)) => false,
            _ => true,
        }
    }

    pub fn try_recv(&mut self) -> Option<Result<Box<dyn PacketVisitor + Send>, std::io::Error>> {
        match self.inbound_packets_rx.try_recv().into() {
            Ok(p) => Some(p),
            Err(_) => None,
        }
    }
}

async fn expect_packet<P: InboundPacket>(reader: &mut BufReader<OwnedReadHalf>) -> Result<P, Error> {
    let id = reader.read_u8().await?;
    if id != P::ID {
        return Err(Error::new(ErrorKind::InvalidInput, format!("expected packet ID {} but got {}", P::ID, id)))
    }

    P::deserialize(reader).await
}
