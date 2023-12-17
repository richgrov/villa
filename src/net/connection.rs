use tokio::{net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}, io::{BufReader, AsyncReadExt, AsyncWriteExt}};

use std::io::{Error, ErrorKind};

use super::packets::{self, InboundPacket, OutboundPacket};

pub struct Connection {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl Connection {
    pub async fn connect(address: &str, username: &str) -> Result<Connection, Error> {
        let stream = TcpStream::connect(address).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::with_capacity(1024, reader);

        writer.write_all(&packets::Handshake {
            username: username.to_owned(),
        }.serialize()?).await?;

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
        }.serialize()?).await?;

        Ok(Connection {
            reader,
            writer,
        })
    }
}

async fn expect_packet<P: InboundPacket>(reader: &mut BufReader<OwnedReadHalf>) -> Result<P, Error> {
    let id = reader.read_u8().await?;
    if id != P::ID {
        return Err(Error::new(ErrorKind::InvalidInput, format!("expected packet ID {} but got {}", P::ID, id)))
    }

    P::deserialize(reader).await
}