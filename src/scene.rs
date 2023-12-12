use winit::dpi::PhysicalPosition;

use crate::gpu::GpuWrapper;

pub trait Scene {
    fn handle_resize(&mut self, gpu: &GpuWrapper, width: f32, height: f32);
    fn handle_mouse_move(&mut self, gpu: &GpuWrapper, position: PhysicalPosition<f32>);
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}
