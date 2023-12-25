use async_trait::async_trait;
use tokio::{io::{BufReader, AsyncReadExt}, net::tcp::OwnedReadHalf};
use zune_inflate::DeflateDecoder;

use crate::world::Block;

use super::serialize::{write_str, read_str, EntityAttributeValue, read_entity_attributes};
use std::{io::{Error, Write, ErrorKind}, collections::HashMap};

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
    fn handle_chat(&mut self, packet: &Chat);
    fn handle_set_time(&mut self, packet: &SetTime);
    fn handle_set_health(&mut self, packet: &SetHealth);
    fn handle_spawn_pos(&mut self, packet: &SpawnPos);
    fn handle_pos(&mut self, packet: &Position);
    fn handle_pos_rot(&mut self, packet: &PosRot);
    fn handle_spawn_item_entity(&mut self, packet: &SpawnItemEntity);
    fn handle_spawn_insentient_entity(&mut self, packet: &SpawnInsentientEntity);
    fn handle_spawn_entity(&mut self, packet: &SpawnEntity);
    fn handle_entity_velocity(&mut self, packet: &EntityVelocity);
    fn handle_remove_entity(&mut self, packet: &RemoveEntity);
    fn handle_move_entity(&mut self, packet: &MoveEntity);
    fn handle_entity_move_rot(&mut self, packet: &EntityMoveRot);
    fn handle_entity_pos_rot(&mut self, packet: &EntityPosRot);
    fn handle_set_entity_health(&mut self, packet: &SetEntityHealth);
    fn handle_update_entity_attributes(&mut self, packet: &UpdateEntityAttributes);
    fn handle_init_chunk(&mut self, packet: &InitChunk);
    fn handle_set_contiguous_blocks(&mut self, packet: &SetContiguousBlocks);
    fn handle_set_blocks(&mut self, packet: &SetBlocks);
    fn handle_set_block(&mut self, packet: &SetBlock);
    fn handle_after_respawn(&mut self, packet: &AfterRespawn);
    fn handle_set_inventory_slot(&mut self, packet: &SetInventorySlot);
    fn handle_set_inventory_items(&mut self, packet: &SetInventoryItems);
    fn handle_disconnect(&mut self, packet: &Disconnect);
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

pub struct Chat {
    pub message: String,
}

id!(Chat, 3);

#[async_trait]
impl InboundPacket for Chat {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(Self {
            message: read_str(reader, 119).await?,
        })
    }
}

impl_visitor!(Chat, handle_chat);

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

pub struct SetHealth {
    pub health: i16,
}

id!(SetHealth, 8);

#[async_trait]
impl InboundPacket for SetHealth {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SetHealth {
            health: reader.read_i16().await?,
        })
    }
}

impl_visitor!(SetHealth, handle_set_health);

pub struct Position {
    pub x: f64,
    pub y: f64,
    pub stance: f64,
    pub z: f64,
    pub grounded: bool,
}

id!(Position, 11);

#[async_trait]
impl InboundPacket for Position {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(Position {
            x: reader.read_f64().await?,
            y: reader.read_f64().await?,
            stance: reader.read_f64().await?,
            z: reader.read_f64().await?,
            grounded: reader.read_u8().await? != 0,
        })
    }
}

impl_visitor!(Position, handle_pos);

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

pub struct SpawnInsentientEntity {
    pub id: i32,
    pub ty: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub shooter: i32,
    pub projectile_velocity: Option<(i16, i16, i16)>,
}

id!(SpawnInsentientEntity, 23);

#[async_trait]
impl InboundPacket for SpawnInsentientEntity {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        let id = reader.read_i32().await?;
        let ty = reader.read_u8().await?;
        let x = reader.read_i32().await?;
        let y = reader.read_i32().await?;
        let z = reader.read_i32().await?;
        let shooter = reader.read_i32().await?;

        Ok(SpawnInsentientEntity {
            id,
            ty,
            x,
            y,
            z,
            shooter,
            projectile_velocity: {
                if shooter > 0 {
                    Some((reader.read_i16().await?, reader.read_i16().await?, reader.read_i16().await?))
                } else {
                    None
                }
            }
        })
    }
}

impl_visitor!(SpawnInsentientEntity, handle_spawn_insentient_entity);

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

pub struct RemoveEntity {
    pub id: i32,
}

id!(RemoveEntity, 29);

#[async_trait]
impl InboundPacket for RemoveEntity {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(RemoveEntity {
            id: reader.read_i32().await?,
        })
    }
}

impl_visitor!(RemoveEntity, handle_remove_entity);

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

pub struct EntityPosRot {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: u8,
    pub pitch: u8,
}

id!(EntityPosRot, 34);

#[async_trait]
impl InboundPacket for EntityPosRot {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(EntityPosRot {
            id: reader.read_i32().await?,
            x: reader.read_i32().await?,
            y: reader.read_i32().await?,
            z: reader.read_i32().await?,
            yaw: reader.read_u8().await?,
            pitch: reader.read_u8().await?,
        })
    }
}

impl_visitor!(EntityPosRot, handle_entity_pos_rot);

pub struct SetEntityHealth {
    pub id: i32,
    pub health: i8,
}

id!(SetEntityHealth, 38);

#[async_trait]
impl InboundPacket for SetEntityHealth {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SetEntityHealth {
            id: reader.read_i32().await?,
            health: reader.read_i8().await?,
        })
    }
}

impl_visitor!(SetEntityHealth, handle_set_entity_health);

pub struct UpdateEntityAttributes {
    pub id: i32,
    pub attributes: HashMap<i8, EntityAttributeValue>,
}

id!(UpdateEntityAttributes, 40);

#[async_trait]
impl InboundPacket for UpdateEntityAttributes {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(UpdateEntityAttributes {
            id: reader.read_i32().await?,
            attributes: read_entity_attributes(reader).await?,
        })
    }
}

impl_visitor!(UpdateEntityAttributes, handle_update_entity_attributes);

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

pub struct SetContiguousBlocks {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    pub x_size: u8,
    pub y_size: u8,
    pub z_size: u8,
    pub data: Vec<u8>,
}

id!(SetContiguousBlocks, 51);

#[async_trait]
impl InboundPacket for SetContiguousBlocks {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(SetContiguousBlocks {
            x: reader.read_i32().await?,
            y: reader.read_i16().await?,
            z: reader.read_i32().await?,
            x_size: reader.read_u8().await?,
            y_size: reader.read_u8().await?,
            z_size: reader.read_u8().await?,
            data: {
                let size = reader.read_i32().await?;
                let mut data = vec![0; size as usize];
                reader.read_exact(&mut data).await?;

                let mut decoder = DeflateDecoder::new(&data);
                decoder.decode_zlib().map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            },
        })
    }
}

impl_visitor!(SetContiguousBlocks, handle_set_contiguous_blocks);

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
        let num_blocks = reader.read_i16().await? as usize;
        let mut positions = Vec::with_capacity(num_blocks);
        for _ in 0..num_blocks {
            positions.push(reader.read_i16().await?);
        }

        let mut types = vec![0; num_blocks];
        reader.read_exact(&mut types).await?;
        let mut data = vec![0; num_blocks];
        reader.read_exact(&mut data).await?;

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

#[derive(Debug)]
pub enum AfterRespawn {
    BedMissing,
    StartRaining,
    StopRaining,
}

id!(AfterRespawn, 70);

#[async_trait]
impl InboundPacket for AfterRespawn {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(match reader.read_i8().await? {
            0 => AfterRespawn::BedMissing,
            1 => AfterRespawn::StartRaining,
            2 => AfterRespawn::StopRaining,
            other => return Err(Error::new(ErrorKind::InvalidInput, format!("{} is not a valid respawn action", other))),
        })
    }
}

impl_visitor!(AfterRespawn, handle_after_respawn);

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

pub struct Disconnect {
    pub message: String,
}

id!(Disconnect, 255);

#[async_trait]
impl InboundPacket for Disconnect {
    async fn deserialize(reader: &mut BufReader<OwnedReadHalf>) -> Result<Self, Error> where Self: Sized {
        Ok(Disconnect {
            message: read_str(reader, 100).await?,
        })
    }
}

impl_visitor!(Disconnect, handle_disconnect);
