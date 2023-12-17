use std::rc::Rc;

use glam::{Mat4, Vec3};
use wgpu::{RenderPipeline, BindGroup};
use winit::{dpi::PhysicalPosition, event::{ElementState, MouseButton, KeyboardInput}};

use crate::{scene::{Scene, NextState}, gpu::GpuWrapper, uniforms::{UniformSpec, UniformStorage}};

use super::{Chunk, chunk::ChunkVertex};

const MOUSE_SENSITIVITY: f32 = 0.01;
const MOVE_SPEED: f32 = 0.1;

const KEY_W: u32 = 17;
const KEY_A: u32 = 30;
const KEY_S: u32 = 31;
const KEY_D: u32 = 32;
const KEY_SHIFT: u32 = 42;
const KEY_SPACE: u32 = 57;

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
    chunk: Chunk,
    resources: Rc<WorldResources>,
    chunk_uniforms: UniformStorage,
    last_cursor_position: Option<PhysicalPosition<f32>>,
    projection: Mat4,

    forward_input: f32,
    left_input: f32,
    up_input: f32,

    camera_pitch: f32,
    camera_yaw: f32,
    camera_x: f32,
    camera_y: f32,
    camera_z: f32,
}

impl World {
    pub fn new(gpu: &GpuWrapper, resources: Rc<WorldResources>) -> World {
        let chunk_uniforms = UniformStorage::new(gpu, "Chunk Uniforms", &[
            (&resources.chunk_uniform_layout, 1, "Chunk Bindings"),
        ]);

        World {
            chunk: Chunk::new(0, 0),
            resources,
            chunk_uniforms,
            last_cursor_position: None,
            projection: Mat4::ZERO,

            forward_input: 0.,
            left_input: 0.,
            up_input: 0.,

            camera_pitch: 0.,
            camera_yaw: 0.,
            camera_x: 0.,
            camera_y: 0.,
            camera_z: -1.,
        }
    }

    fn update_position(&mut self, gpu: &GpuWrapper) {
        let view = Mat4::from_rotation_x(self.camera_pitch)
            * Mat4::from_rotation_y(self.camera_yaw)
            * Mat4::from_translation(Vec3::new(-self.camera_x, -self.camera_y, -self.camera_z));
        let model = Mat4::from_translation(Vec3::new(0., 0., 0.));

        self.chunk_uniforms.set_element(0, 0, (self.projection * view * model).to_cols_array());
        self.chunk_uniforms.update(gpu);
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
        self.chunk.rebuild_mesh(gpu);
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
        self.camera_x -= self.camera_yaw.sin() * self.forward_input * MOVE_SPEED;
        self.camera_z += self.camera_yaw.cos() * self.forward_input * MOVE_SPEED;

        self.camera_x -= self.camera_yaw.cos() * self.left_input * MOVE_SPEED;
        self.camera_z -= self.camera_yaw.sin() * self.left_input * MOVE_SPEED;

        self.camera_y += 0.1 * self.up_input;

        self.update_position(gpu);
        NextState::Continue
    }

    fn draw_3d<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.resources.pipeline);
        render_pass.set_bind_group(0, &self.resources.terrain_texture, &[]);
        render_pass.set_bind_group(1, self.chunk_uniforms.bind_group(0), &[0]);
        if let Some(mesh) = self.chunk.mesh() {
            mesh.bind(render_pass);
            mesh.draw(render_pass);
        }
    }
}