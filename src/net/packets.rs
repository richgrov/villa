use async_trait::async_trait;
use tokio::{io::{BufReader, AsyncReadExt}, net::tcp::OwnedReadHalf};

use super::serialize::{write_str, read_str, EntityAttributeValue, read_entity_attributes};
use std::{io::{Error, Write}, collections::HashMap};

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
    fn handle_set_time(&mut self, packet: &SetTime);
    fn handle_spawn_pos(&mut self, packet: &SpawnPos);
    fn handle_pos_rot(&mut self, packet: &PosRot);
    fn handle_spawn_entity(&mut self, packet: &SpawnEntity);
    fn handle_entity_velocity(&mut self, packet: &EntityVelocity);
    fn handle_init_chunk(&mut self, packet: &InitChunk);
}

pub trait PacketVisitor<H: PacketHandler> {
    fn visit(&self, handler: &mut H);
}

macro_rules! id {
    ($ty:ty, $id:literal) => {
        impl Packet for $ty {
            const ID: u8 = $id;
        }
    };
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

id!(Login, 1);

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

id!(Handshake, 2);

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

pub struct SetTime {
    pub time: i64,
}

id!(SetTime, 4);

#[async_trait]
impl InboundPacket for SetTime {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SetTime {
            time: reader.read_i64().await?,
        })
    }
}

impl_visitor!(SetTime, handle_set_time);

pub struct SpawnPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

id!(SpawnPos, 6);

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

pub struct PosRot {
    pub x: f64,
    pub y: f64,
    pub stance: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub grounded: bool,
}

id!(PosRot, 13);

#[async_trait]
impl InboundPacket for PosRot {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(PosRot {
            x: reader.read_f64().await?,
            y: reader.read_f64().await?,
            stance: reader.read_f64().await?,
            z: reader.read_f64().await?,
            yaw: reader.read_f32().await?,
            pitch: reader.read_f32().await?,
            grounded: reader.read_u8().await? != 0,
        })
    }
}

impl_visitor!(PosRot, handle_pos_rot);

pub struct SpawnEntity {
    pub id: i32,
    pub ty: i8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: i8,
    pub pitch: i8,
    pub attributes: HashMap<i8, EntityAttributeValue>,
}

id!(SpawnEntity, 24);

#[async_trait]
impl InboundPacket for SpawnEntity {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SpawnEntity {
            id: reader.read_i32().await?,
            ty: reader.read_i8().await?,
            x: reader.read_i32().await?,
            y: reader.read_i32().await?,
            z: reader.read_i32().await?,
            yaw: reader.read_i8().await?,
            pitch: reader.read_i8().await?,
            attributes: read_entity_attributes(reader).await?,
        })
    }
}

impl_visitor!(SpawnEntity, handle_spawn_entity);

pub struct EntityVelocity {
    pub id: i32,
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

id!(EntityVelocity, 28);

#[async_trait]
impl InboundPacket for EntityVelocity {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(EntityVelocity {
            id: reader.read_i32().await?,
            x: reader.read_i16().await?,
            y: reader.read_i16().await?,
            z: reader.read_i16().await?,
        })
    }
}

impl_visitor!(EntityVelocity, handle_entity_velocity);

pub struct InitChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub init: bool,
}

id!(InitChunk, 50);

#[async_trait]
impl InboundPacket for InitChunk {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(InitChunk {
            chunk_x: reader.read_i32().await?,
            chunk_z: reader.read_i32().await?,
            init: reader.read_u8().await? != 0,
        })
    }
}

impl_visitor!(InitChunk, handle_init_chunk);
