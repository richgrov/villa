use std::{rc::Rc, collections::HashMap};

use glam::{Mat4, Vec3};
use slab::Slab;
use wgpu::{RenderPipeline, BindGroup};
use winit::{dpi::PhysicalPosition, event::{ElementState, MouseButton, KeyboardInput}};

use crate::{scene::{Scene, NextState}, gpu::GpuWrapper, uniforms::{UniformSpec, UniformStorage}, net::{packets::{self, PacketHandler, OutboundPacket}, Connection}, world::chunk};

use super::{Chunk, chunk::ChunkVertex, Block};

const MOUSE_SENSITIVITY: f32 = 0.01;
const MOVE_SPEED: f32 = 0.1;

const KEY_W: u32 = 17;
const KEY_A: u32 = 30;
const KEY_S: u32 = 31;
const KEY_D: u32 = 32;
const KEY_SHIFT: u32 = 42;
const KEY_SPACE: u32 = 57;

const PLAYER_EYE_HEIGHT: f64 = 1.62;
const CHUNK_VIEW_RADIUS: i32 = 8;
// CHUNK_VIEW_RADIUS on each side and one chunk in the middle
const TOTAL_VIEWABLE_CHUNKS: i32 = (CHUNK_VIEW_RADIUS*2 + 1).pow(2);

pub struct WorldResources {
    chunk_uniform_layout: UniformSpec,
    pipeline: RenderPipeline,
    terrain_texture: BindGroup,
}

impl WorldResources {
    pub fn new(gpu: &GpuWrapper) -> WorldResources {
        let terrain_image = image::load_from_memory(include_bytes!("../../res/terrain.png")).unwrap();
        let terrain_texture = gpu.create_texture(&terrain_image);

        let chunk_uniform_layout = UniformSpec::new::<Mat4>(gpu, "Chunk Uniform Layout", wgpu::ShaderStages::VERTEX);
        let pipeline = gpu.create_pipeline::<ChunkVertex>(
            "Chunk Pipeline",
            include_str!("../../res/chunk.wgsl"),
            &[gpu.generic_texture_layout(), chunk_uniform_layout.layout()],
            true,
        );

        WorldResources {
            chunk_uniform_layout,
            pipeline,
            terrain_texture,
        }
    }
}

pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
    resources: Rc<WorldResources>,
    chunk_uniforms: UniformStorage,
    rendered_chunks: Slab<(i32, i32)>,
    last_cursor_position: Option<PhysicalPosition<f32>>,
    projection: Mat4,
    connection: Connection,
    disconnected: bool,

    forward_input: f32,
    left_input: f32,
    up_input: f32,

    camera_pitch: f32,
    camera_yaw: f32,
    camera_x: f32,
    camera_y: f32,
    camera_z: f32,
    position_initialized: bool,
}

impl World {
    pub fn new(gpu: &GpuWrapper, resources: Rc<WorldResources>, connection: Connection) -> World {
        let chunk_uniforms = UniformStorage::new(gpu, "Chunk Uniforms", &[
            (&resources.chunk_uniform_layout, TOTAL_VIEWABLE_CHUNKS as usize, "Chunk Bindings"),
        ]);

        World {
            chunks: HashMap::with_capacity(TOTAL_VIEWABLE_CHUNKS as usize),
            resources,
            chunk_uniforms,
            rendered_chunks: Slab::with_capacity(TOTAL_VIEWABLE_CHUNKS as usize),
            last_cursor_position: None,
            projection: Mat4::ZERO,
            connection,
            disconnected: false,

            forward_input: 0.,
            left_input: 0.,
            up_input: 0.,

            camera_pitch: 0.,
            camera_yaw: 0.,
            camera_x: 0.,
            camera_y: 0.,
            camera_z: 0.,
            position_initialized: false,
        }
    }

    fn update_position(&mut self, gpu: &GpuWrapper) {
        let view = Mat4::from_rotation_x(self.camera_pitch)
            * Mat4::from_rotation_y(self.camera_yaw)
            * Mat4::from_translation(Vec3::new(-self.camera_x, -self.camera_y, -self.camera_z));

        for (uniform_index, chunk_pos) in &self.rendered_chunks {
            let chunk = self.chunks.get(chunk_pos).unwrap();
            self.chunk_uniforms.set_element(0, uniform_index, (self.projection * view * chunk.transform()).to_cols_array());
        }
        self.chunk_uniforms.update(gpu);
    }

    fn update_viewable_chunks(&mut self) {
        for chunk_pos in self.rendered_chunks.drain() {
            self.chunks.get_mut(&chunk_pos).unwrap().uniform_offset = u32::MAX;
        }

        let (chunk_x, chunk_z) = chunk::to_chunk_pos(self.camera_x as i32, self.camera_z as i32);
        for x in (chunk_x-CHUNK_VIEW_RADIUS)..=(chunk_x+CHUNK_VIEW_RADIUS) {
            for z in (chunk_z-CHUNK_VIEW_RADIUS)..=(chunk_z+CHUNK_VIEW_RADIUS) {
                if let Some(chunk) = self.chunks.get_mut(&(x, z)) {
                    let uniform_index = self.rendered_chunks.insert((x, z));
                    chunk.uniform_offset = self.resources.chunk_uniform_layout.offset_of(uniform_index);
                }
            }
        }
    }

    fn get_or_init_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &mut Chunk {
        self.chunks.entry((chunk_x, chunk_z)).or_insert_with(|| {
            let (cam_chunk_x, cam_chunk_z) = chunk::to_chunk_pos(self.camera_x as i32, self.camera_z as i32);
            let is_viewable = (chunk_x - cam_chunk_x).abs() <= CHUNK_VIEW_RADIUS && (chunk_z - cam_chunk_z).abs() <= CHUNK_VIEW_RADIUS;

            if is_viewable {
                if self.rendered_chunks.len() >= self.rendered_chunks.capacity() {
                    panic!("tried to add viewable chunk at {}, {} when all rendered slots are taken", chunk_x, chunk_z);
                }

                let uniform_index = self.rendered_chunks.insert((chunk_x, chunk_z));
                let uniform_offset = self.resources.chunk_uniform_layout.offset_of(uniform_index);
                Chunk::new(chunk_x, chunk_z, uniform_offset, Some(uniform_index))
            } else {
                // ::MAX will cause a panic if this chunk is accidentally attempted to be
                // rendered
                Chunk::new(chunk_x, chunk_z, u32::MAX, None)
            }
        })
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let chunk_pos = chunk::to_chunk_pos(x, z);
        let chunk = self.get_or_init_chunk(chunk_pos.0, chunk_pos.1);

        let (relative_x, relative_z) = chunk::world_to_chunk_relative(x, z);
        chunk.set_block(relative_x as usize, y as usize, relative_z as usize, block);
    }

    fn process_packets(&mut self) {
        if self.disconnected {
            return
        }

        while let Some(packet) = self.connection.try_recv() {
            match packet {
                Ok(p) => p.visit(self),
                Err(e) => {
                    eprintln!("connection error: {}", e);
                    self.disconnected = true;
                }
            }
        }
    }

    fn queue_packet<P: OutboundPacket>(&self, packet: &P) {
        if self.disconnected {
            return
        }

        if !self.connection.queue_packet(packet) {
            eprintln!("packet queue is full!");
        }
    }
}

impl Scene for World {
    fn handle_resize(&mut self, _gpu: &crate::gpu::GpuWrapper, width: f32, height: f32) {
        self.projection = Mat4::perspective_lh((70f32).to_radians(), width / height, 0.01, 1000.);
    }

    fn handle_mouse_move(&mut self, gpu: &crate::gpu::GpuWrapper, position: PhysicalPosition<f32>) {
        if let Some(last_pos) = self.last_cursor_position {
            self.camera_pitch -= (last_pos.y - position.y) * MOUSE_SENSITIVITY;
            self.camera_yaw += (last_pos.x - position.x) * MOUSE_SENSITIVITY;
            self.update_position(gpu);
        }
        self.last_cursor_position = Some(position);
    }

    fn handle_click(&mut self, gpu: &GpuWrapper, _state: ElementState, _button: MouseButton) -> NextState {
        NextState::Continue
    }

    fn handle_key_input(&mut self, _gpu: &GpuWrapper, key: KeyboardInput) {
        let factor = match key.state {
            ElementState::Pressed => 1.,
            ElementState::Released => 0.,
        };

        match key.scancode {
            KEY_W => self.forward_input = factor,
            KEY_S => self.forward_input = -factor,
            KEY_A => self.left_input = factor,
            KEY_D => self.left_input = -factor,
            KEY_SHIFT => self.up_input = -factor,
            KEY_SPACE => self.up_input = factor,
            _ => return,
        }
    }

    fn update(&mut self, gpu: &GpuWrapper) -> NextState {
        self.process_packets();

        self.camera_x -= self.camera_yaw.sin() * self.forward_input * MOVE_SPEED;
        self.camera_z += self.camera_yaw.cos() * self.forward_input * MOVE_SPEED;

        self.camera_x -= self.camera_yaw.cos() * self.left_input * MOVE_SPEED;
        self.camera_z -= self.camera_yaw.sin() * self.left_input * MOVE_SPEED;

        self.camera_y += 0.1 * self.up_input;

        self.update_position(gpu);

        for chunk in self.chunks.values_mut() {
            if chunk.needs_mesh_rebuild() {
                chunk.rebuild_mesh(gpu);
            }
        }

        if self.position_initialized {
            self.queue_packet(&packets::PosRot {
                x: self.camera_x as f64,
                y: self.camera_y as f64,
                z: self.camera_z as f64,
                stance: self.camera_y as f64 + PLAYER_EYE_HEIGHT,
                yaw: self.camera_yaw,
                pitch: self.camera_pitch,
                grounded: true,
            });
        }

        NextState::Continue
    }

    fn draw_3d<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.resources.pipeline);
        render_pass.set_bind_group(0, &self.resources.terrain_texture, &[]);
        for (_, chunk_pos) in &self.rendered_chunks {
            let chunk = self.chunks.get(&chunk_pos).unwrap();
            if let Some(mesh) = chunk.mesh() {
                render_pass.set_bind_group(1, self.chunk_uniforms.bind_group(0), &[chunk.uniform_offset]);
                mesh.bind(render_pass);
                mesh.draw(render_pass);
            }
        }
    }
}

impl PacketHandler for World {
    fn handle_login(&mut self, packet: &packets::Login) {
        println!("Seed: {}, Dimension: {}", packet.seed, packet.dimension);
    }

    fn handle_chat(&mut self, packet: &packets::Chat) {
        println!("Chat: {}", packet.message);
    }

    fn handle_set_time(&mut self, packet: &packets::SetTime) {
        println!("World time: {}", packet.time);
    }

    fn handle_set_entity_item(&mut self, _packet:  &packets::SetEntityItem) {
    }

    fn handle_set_health(&mut self, packet: &packets::SetHealth) {
        println!("Health changed to {}", packet.health);
    }

    fn handle_spawn_pos(&mut self, packet: &packets::SpawnPos) {
        println!("Spawn pos: {}, {}, {}", packet.x, packet.y, packet.z);
    }

    fn handle_pos(&mut self, packet: &packets::Position) {
        self.camera_x = packet.x as f32;
        self.camera_y = (packet.y - PLAYER_EYE_HEIGHT) as f32;
        self.camera_z = packet.z as f32;
        self.update_viewable_chunks();
        self.queue_packet(packet);
        self.position_initialized = true;
    }

    fn handle_pos_rot(&mut self, packet: &packets::PosRot) {
        self.camera_x = packet.x as f32;
        self.camera_y = (packet.y - PLAYER_EYE_HEIGHT) as f32;
        self.camera_z = packet.z as f32;
        self.update_viewable_chunks();
        self.queue_packet(packet);
        self.position_initialized = true;
    }

    fn handle_spawn_player(&mut self, packet: &packets::SpawnPlayer) {
        println!("Player {} at {}, {}, {}", packet.name, packet.x, packet.y, packet.z);
    }

    fn handle_spawn_item_entity(&mut self, packet: &packets::SpawnItemEntity) {
        println!("Item on ground at {}, {}, {}: {}", packet.x, packet.y, packet.z, packet.item_id);
    }

    fn handle_spawn_insentient_entity(&mut self, _packet: &packets::SpawnInsentientEntity) {
    }

    fn handle_spawn_entity(&mut self, _packet: &packets::SpawnEntity) {
    }

    fn handle_entity_velocity(&mut self, _packet: &packets::EntityVelocity) {
    }

    fn handle_remove_entity(&mut self, _packet: &packets::RemoveEntity) {
    }

    fn handle_move_entity(&mut self, _packet: &packets::MoveEntity) {
    }

    fn handle_entity_move_rot(&mut self, _packet: &packets::EntityMoveRot) {
    }

    fn handle_entity_pos_rot(&mut self, _packet: &packets::EntityPosRot) {
    }

    fn handle_set_entity_health(&mut self, _packet: &packets::SetEntityHealth) {
    }

    fn handle_init_chunk(&mut self, packet: &packets::InitChunk) {
        if packet.init {
            return
        }

        if let Some(chunk) = self.chunks.remove(&(packet.chunk_x, packet.chunk_z)) {
            if let Some(index) = chunk.uniform_index {
                self.rendered_chunks.remove(index);
            }
        }
    }

    fn handle_set_contiguous_blocks(&mut self, packet: &packets::SetContiguousBlocks) {
        let start_chunk_pos = chunk::to_chunk_pos(packet.x, packet.z);
        let end_chunk_pos = chunk::to_chunk_pos(packet.x + packet.x_size - 1, packet.z + packet.z_size - 1);

        // The Notchian server should never set contiguous blocks between chunk boundaries, but the
        // protocol allows for modified servers to potentially do it. We fall back an a
        // non-optimized algorithm in case of this
        if start_chunk_pos != end_chunk_pos {
            let mut index = 0;
            for x in 0..packet.x_size {
                for z in 0..packet.z_size {
                    for y in 0..packet.y_size {
                        self.set_block(packet.x + x, packet.y + y, packet.z + z, packet.blocks[index]);
                        index += 1;
                    }
                }
            }
            return
        }

        let chunk = self.get_or_init_chunk(start_chunk_pos.0, start_chunk_pos.1);
        let (x_offset, z_offset) = chunk::world_to_chunk_relative(packet.x, packet.z);
        let mut index = 0;
        for x in 0..packet.x_size {
            for z in 0..packet.z_size {
                for y in 0..packet.y_size {
                    chunk.set_block((x + x_offset) as usize, y as usize, (z + z_offset) as usize, packet.blocks[index]);
                    index += 1;
                }
            }
        }
    }

    fn handle_set_blocks(&mut self, packet: &packets::SetBlocks) {
        let chunk = self.get_or_init_chunk(packet.chunk_x, packet.chunk_z);

        for (i, (x, y, z)) in packet.positions.iter().enumerate() {
            let ty = packet.types[i];
            let data = packet.data[i];
            let block = Block::read(ty, data).unwrap_or(Block::Stone); // TODO
            chunk.set_block(*x as usize, *y as usize, *z as usize, block);
        }
    }

    fn handle_set_block(&mut self, packet: &packets::SetBlock) {
        self.set_block(packet.x, packet.y as i32, packet.z, packet.block);
    }

    fn handle_update_entity_attributes(&mut self, packet: &packets::UpdateEntityAttributes) {
        println!("Change {} attributes on entity {}", packet.attributes.len(), packet.id);
    }

    fn handle_after_respawn(&mut self, packet: &packets::AfterRespawn) {
        println!("After respawn: {:?}", packet);
    }

    fn handle_set_inventory_slot(&mut self, packet: &packets::SetInventorySlot) {
        println!("Inventory {}, slot {}: {:?}", packet.inventory_id, packet.slot, packet.item.map(|i| i.0))
    }

    fn handle_set_inventory_items(&mut self, packet: &packets::SetInventoryItems) {
        println!("Inventory {}: {} items", packet.inventory_id, packet.items.len());
    }

    fn handle_statistic(&mut self, _packet: &packets::Statistic) {
    }

    fn handle_disconnect(&mut self, packet: &packets::Disconnect) {
        println!("Disconnected: {}", packet.message);
    }
}
