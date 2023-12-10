use std::rc::Rc;

use wgpu::RenderPass;
use winit::dpi::PhysicalPosition;

use crate::gpu::GpuWrapper;

use super::{GuiResources, GuiSpec, Gui};

pub struct TitleGui {
    pub gui: Gui,
    gui_resources: Rc<GuiResources>,
    singleplayer: usize,
    multiplayer: usize,
    options: usize,
    quit: usize,
}

impl TitleGui {
    pub fn new(gpu: &GpuWrapper, gui_renderer: Rc<GuiResources>) -> TitleGui {
        let mut gui = GuiSpec::new();
        let singleplayer = gui.button("Singleplayer");
        let multiplayer = gui.button("Multiplayer");
        let options = gui.button("Options");
        let quit = gui.button("Quit");

        TitleGui {
            gui: gui_renderer.build_gui(gpu, &gui),
            gui_resources: gui_renderer,
            singleplayer,
            multiplayer,
            options,
            quit,
        }
    }

    pub fn handle_resize(&mut self, gpu: &GpuWrapper, width: f32, height: f32) {
        self.gui.set_button_pos(self.singleplayer, width / 2., height * 0.8);
        self.gui.set_button_pos(self.multiplayer, width / 2., height * 0.6);
        self.gui.set_button_pos(self.options, width / 2., height * 0.4);
        self.gui.set_button_pos(self.quit, width / 2., height * 0.2);
        self.gui.resize(gpu);
    }

    pub fn handle_mouse_move(&mut self, gpu: &GpuWrapper, position: PhysicalPosition<f32>) {
        self.gui.mouse_moved(gpu, position);
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.gui.render(render_pass, &self.gui_resources);
    }
}
