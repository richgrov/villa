use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseButton},
};

use crate::gpu::GpuWrapper;

pub trait Scene {
    fn handle_resize(&mut self, gpu: &GpuWrapper, width: f32, height: f32);
    fn handle_mouse_move(&mut self, gpu: &GpuWrapper, position: PhysicalPosition<f32>);
    fn handle_click(
        &mut self,
        gpu: &GpuWrapper,
        state: ElementState,
        button: MouseButton,
    ) -> NextState;
    fn handle_key_input(&mut self, gpu: &GpuWrapper, key: KeyboardInput);
    fn update(&mut self, gpu: &GpuWrapper) -> NextState;
    fn draw_ui<'a>(&'a self, _render_pass: &mut wgpu::RenderPass<'a>) {}
    fn draw_3d<'a>(&'a self, _render_pass: &mut wgpu::RenderPass<'a>) {}
}

pub enum NextState {
    Continue,
    ChangeScene(Box<dyn Scene>),
    Exit,
}
