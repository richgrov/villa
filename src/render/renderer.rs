use winit::window::Window;

use super::{font::FontRenderer, gpu::GpuWrapper};

pub struct Renderer {
    gpu: GpuWrapper,
    font: FontRenderer,
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let gpu = GpuWrapper::new(window).await;
        let font = FontRenderer::new(&gpu);

        Renderer {
            gpu,
            font,
        }
    }

    pub fn gpu(&self) -> &GpuWrapper {
        &self.gpu
    }

    pub fn gpu_mut(&mut self) -> &mut GpuWrapper {
        &mut self.gpu
    }

    pub fn font(&self) -> &FontRenderer {
        &self.font
    }
}
