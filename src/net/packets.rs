use async_trait::async_trait;
use tokio::{io::{BufReader, AsyncReadExt}, net::tcp::OwnedReadHalf};

use crate::world::Block;

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
    fn handle_spawn_item_entity(&mut self, packet: &SpawnItemEntity);
    fn handle_spawn_entity(&mut self, packet: &SpawnEntity);
    fn handle_entity_velocity(&mut self, packet: &EntityVelocity);
    fn handle_move_entity(&mut self, packet: &MoveEntity);
    fn handle_entity_move_rot(&mut self, packet: &EntityMoveRot);
    fn handle_init_chunk(&mut self, packet: &InitChunk);
    fn handle_set_blocks(&mut self, packet: &SetBlocks);
    fn handle_set_block(&mut self, packet: &SetBlock);
    fn handle_set_inventory_slot(&mut self, packet: &SetInventorySlot);
    fn handle_set_inventory_items(&mut self, packet: &SetInventoryItems);
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

pub struct SpawnItemEntity {
    pub entity_id: i32,
    pub item_id: i16,
    pub count: i8,
    pub data: i16,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub vel_x: i8,
    pub vel_y: i8,
    pub vel_z: i8,
}

id!(SpawnItemEntity, 21);

#[async_trait]
impl InboundPacket for SpawnItemEntity {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SpawnItemEntity {
            entity_id: reader.read_i32().await?,
            item_id: reader.read_i16().await?,
            count: reader.read_i8().await?,
            data: reader.read_i16().await?,
            x: reader.read_i32().await?,
            y: reader.read_i32().await?,
            z: reader.read_i32().await?,
            vel_x: reader.read_i8().await?,
            vel_y: reader.read_i8().await?,
            vel_z: reader.read_i8().await?,
        })
    }
}

impl_visitor!(SpawnItemEntity, handle_spawn_item_entity);

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

pub struct MoveEntity {
    pub id: i32,
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

id!(MoveEntity, 31);

#[async_trait]
impl InboundPacket for MoveEntity {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(MoveEntity {
            id: reader.read_i32().await?,
            x: reader.read_i8().await?,
            y: reader.read_i8().await?,
            z: reader.read_i8().await?,
        })
    }
}

impl_visitor!(MoveEntity, handle_move_entity);

pub struct EntityMoveRot {
    pub id: i32,
    pub x: i8,
    pub y: i8,
    pub z: i8,
    pub yaw: i8,
    pub pitch: i8,
}

id!(EntityMoveRot, 33);

#[async_trait]
impl InboundPacket for EntityMoveRot {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(EntityMoveRot {
            id: reader.read_i32().await?,
            x: reader.read_i8().await?,
            y: reader.read_i8().await?,
            z: reader.read_i8().await?,
            yaw: reader.read_i8().await?,
            pitch: reader.read_i8().await?,
        })
    }
}

impl_visitor!(EntityMoveRot, handle_entity_move_rot);

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

pub struct SetBlocks {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub positions: Vec<i16>,
    pub types: Vec<u8>,
    pub data: Vec<u8>,
}

id!(SetBlocks, 52);

#[async_trait]
impl InboundPacket for SetBlocks {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        let chunk_x = reader.read_i32().await?;
        let chunk_z = reader.read_i32().await?;
        let num_blocks = reader.read_u16().await? as usize;
        let mut positions = Vec::with_capacity(num_blocks);
        for _ in 0..num_blocks {
            positions.push(reader.read_i16().await?);
        }

        let mut types = Vec::with_capacity(num_blocks);
        reader.read(&mut types).await?;
        let mut data = Vec::with_capacity(num_blocks);
        reader.read(&mut data).await?;

        Ok(SetBlocks {
            chunk_x,
            chunk_z,
            positions,
            types,
            data,
        })
    }
}

impl_visitor!(SetBlocks, handle_set_blocks);

pub struct SetBlock {
    pub x: i32,
    pub y: u8,
    pub z: i32,
    pub block: Block,
}

id!(SetBlock, 53);

#[async_trait]
impl InboundPacket for SetBlock {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SetBlock {
            x: reader.read_i32().await?,
            y: reader.read_u8().await?,
            z: reader.read_i32().await?,
            block: Block::read(reader.read_u8().await?, reader.read_u8().await?).unwrap_or(Block::Air),
        })
    }
}

impl_visitor!(SetBlock, handle_set_block);

pub struct SetInventorySlot {
    pub inventory_id: i8,
    pub slot: i16,
    pub item: Option<(i16, i8, i16)>,
}

id!(SetInventorySlot, 103);

#[async_trait]
impl InboundPacket for SetInventorySlot {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SetInventorySlot {
            inventory_id: reader.read_i8().await?,
            slot: reader.read_i16().await?,
            item: {
                let id = reader.read_i16().await?;
                if id < 0 {
                    None
                } else {
                    Some((id, reader.read_i8().await?, reader.read_i16().await?))
                }
            },
        })
    }
}

impl_visitor!(SetInventorySlot, handle_set_inventory_slot);

pub struct SetInventoryItems {
    pub inventory_id: i8,
    pub items: Vec<Option<(i16, i8, i16)>>,
}

id!(SetInventoryItems, 104);

#[async_trait]
impl InboundPacket for SetInventoryItems {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        let inventory_id = reader.read_i8().await?;
        let num_items = reader.read_i16().await?;
        let mut items = Vec::with_capacity(num_items as usize);
        for _ in 0..num_items {
            let id = reader.read_i16().await?;
            if id < 0 {
                items.push(None);
                continue
            }

            let count = reader.read_i8().await?;
            let data = reader.read_i16().await?;
            items.push(Some((id, count, data)));
        }

        Ok(SetInventoryItems {
            inventory_id,
            items,
        })
    }
}

impl_visitor!(SetInventoryItems, handle_set_inventory_items);
