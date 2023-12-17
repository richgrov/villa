use async_trait::async_trait;
use tokio::{io::{BufReader, AsyncReadExt}, net::tcp::OwnedReadHalf};

use super::serialize::{write_str, read_str};
use std::io::{Error, Write};

pub const PROTOCOL_VERSION: i32 = 14;

pub trait Packet {
    const ID: u8;
}

#[async_trait]
pub trait InboundPacket: Packet {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized;
}

pub trait OutboundPacket: Packet {
    fn serialize(&self) -> Result<Vec<u8>, Error>;
}

pub trait PacketHandler {
    fn handle_login(&mut self, packet: &Login);
    fn handle_spawn_pos(&mut self, packet: &SpawnPos);
}

pub trait PacketVisitor<H: PacketHandler> {
    fn visit(&self, handler: &mut H);
}

macro_rules! impl_visitor {
    ($ty:ty, $func:ident) => {
        impl<H: PacketHandler> PacketVisitor<H> for $ty {
            fn visit(&self, handler: &mut H) {
                handler.$func(self);
            }
        }
    };
}

pub struct Login {
    pub protocol_version: i32,
    pub username: String,
    pub seed: i64,
    pub dimension: i8,
}

impl Packet for Login {
    const ID: u8 = 1;
}

#[async_trait]
impl InboundPacket for Login {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(Login {
            protocol_version: reader.read_i32().await?,
            username: read_str(reader, 16).await?,
            seed: reader.read_i64().await?,
            dimension: reader.read_i8().await?,
        })
    }
}

impl OutboundPacket for Login {
    fn serialize(&self) -> Result<Vec<u8>, Error> {
        let mut data = Vec::with_capacity(24);
        data.push(Self::ID);
        let _ = data.write(&self.protocol_version.to_be_bytes());
        write_str(&mut data, &self.username)?;
        let _ = data.write(&self.seed.to_be_bytes());
        let _ = data.write(&[self.dimension as u8]);
        Ok(data)
    }
}

impl_visitor!(Login, handle_login);

pub struct Handshake {
    pub username: String,
}

impl Packet for Handshake {
    const ID: u8 = 2;
}

#[async_trait]
impl InboundPacket for Handshake {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(Handshake {
            username: read_str(reader, 16).await?,
        })
    }
}

impl OutboundPacket for Handshake {
    fn serialize(&self) -> Result<Vec<u8>, Error> {
        let mut data = Vec::with_capacity(16);
        data.push(Self::ID);
        write_str(&mut data, &self.username)?;
        Ok(data)
    }
}

pub struct SpawnPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Packet for SpawnPos {
    const ID: u8 = 6;
}

#[async_trait]
impl InboundPacket for SpawnPos {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SpawnPos {
            x: reader.read_i32().await?,
            y: reader.read_i32().await?,
            z: reader.read_i32().await?,
        })
    }
}

impl_visitor!(SpawnPos, handle_spawn_pos);
