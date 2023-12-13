use std::rc::Rc;

use glam::{Mat4, Vec3};
use wgpu::RenderPipeline;
use winit::{dpi::PhysicalPosition, event::{ElementState, MouseButton}};

use crate::{scene::{Scene, NextState}, gpu::GpuWrapper, uniforms::{UniformSpec, UniformStorage}};

use super::{Chunk, chunk::ChunkVertex};

const MOUSE_SENSITIVITY: f32 = 0.3;

pub struct WorldResources {
    chunk_uniform_layout: UniformSpec,
    pipeline: RenderPipeline,
}

impl WorldResources {
    pub fn new(gpu: &GpuWrapper) -> WorldResources {
        let chunk_uniform_layout = UniformSpec::new::<Mat4>(gpu, "Chunk Uniform Layout", wgpu::ShaderStages::VERTEX);
        let pipeline = gpu.create_pipeline::<ChunkVertex>("Chunk Pipeline", include_str!("../../res/chunk.wgsl"), &[chunk_uniform_layout.layout()]);

        WorldResources {
            chunk_uniform_layout,
            pipeline,
        }
    }
}

pub struct World {
    chunk: Chunk,
    resources: Rc<WorldResources>,
    chunk_uniforms: UniformStorage,
    last_cursor_position: Option<PhysicalPosition<f32>>,
    projection: Mat4,

    camera_pitch: f32,
    camera_yaw: f32,
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

            camera_pitch: 0.,
            camera_yaw: 0.,
        }
    }
}

impl Scene for World {
    fn handle_resize(&mut self, gpu: &crate::gpu::GpuWrapper, width: f32, height: f32) {
        self.projection = Mat4::perspective_lh((70f32).to_radians(), width / height, 0., 1000.);
    }

    fn handle_mouse_move(&mut self, gpu: &crate::gpu::GpuWrapper, position: PhysicalPosition<f32>) {
        if let Some(last_pos) = self.last_cursor_position {
            self.camera_pitch -= (last_pos.y - position.y) * MOUSE_SENSITIVITY;
            self.camera_yaw += (last_pos.x - position.x) * MOUSE_SENSITIVITY;

            let view = Mat4::from_rotation_x(self.camera_pitch.to_radians())
                * Mat4::from_rotation_y(self.camera_yaw.to_radians())
                * Mat4::from_translation(Vec3::new(0., 0., 1.));
            let model = Mat4::from_translation(Vec3::new(0., 0., 0.));

            self.chunk_uniforms.set_element(0, 0, (self.projection * view * model).to_cols_array());
            self.chunk_uniforms.update(gpu);
        }
        self.last_cursor_position = Some(position);
    }

    fn handle_click(&mut self, gpu: &GpuWrapper, state: ElementState, button: MouseButton) -> NextState {
        self.chunk.build_mesh(gpu);
        NextState::Continue
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.resources.pipeline);
        render_pass.set_bind_group(0, self.chunk_uniforms.bind_group(0), &[0]);
        if let Some(mesh) = self.chunk.mesh() {
            mesh.bind(render_pass);
            mesh.draw(render_pass);
        }
    }
}
