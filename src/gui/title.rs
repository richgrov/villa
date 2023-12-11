use std::rc::Rc;

use wgpu::RenderPass;
use winit::dpi::PhysicalPosition;

use crate::gpu::GpuWrapper;

use super::{GuiResources, GuiSpec, Gui, render::SpriteVertex};

const LOGO_ASPECT: f32 = (155.+119.) / 44.;
const LOGO_SPLIT: f32 = 155. / (155.+119.);
const LOGO_VERTICES: [SpriteVertex; 8] = [
    SpriteVertex {
        pos: [0., 0.],
        uv: [0., 44./256.],
    },
    SpriteVertex {
        pos: [LOGO_SPLIT, 0.],
        uv: [155./256., 44./256.],
    },

    SpriteVertex {
        pos: [LOGO_SPLIT, 1.],
        uv: [155./256., 0.],
    },
    SpriteVertex {
        pos: [0., 1.],
        uv: [0., 0.],
    },
    SpriteVertex {
        pos: [LOGO_SPLIT, 0.],
        uv: [0., 89./256.],
    },
    SpriteVertex {
        pos: [1., 0.],
        uv: [119./256., 89./256.],
    },
    SpriteVertex {
        pos: [1., 1.],
        uv: [119./256., 45./256.],
    },
    SpriteVertex {
        pos: [LOGO_SPLIT, 1.],
        uv: [0., 45./256.],
    },
];

const LOGO_INDICES: [u16; 12] = [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

pub struct TitleGui {
    pub gui: Gui,
    gui_resources: Rc<GuiResources>,
    singleplayer: usize,
    multiplayer: usize,
    options: usize,
    quit: usize,
    logo: usize,
}

impl TitleGui {
    pub fn new(gpu: &GpuWrapper, gui_renderer: Rc<GuiResources>) -> TitleGui {
        let mut gui = GuiSpec::new();
        let singleplayer = gui.button("Singleplayer");
        let multiplayer = gui.button("Multiplayer");
        let options = gui.button("Options");
        let quit = gui.button("Quit");

        let logo_image = image::load_from_memory(include_bytes!("../../res/mclogo.png")).unwrap();
        let logo_texture = gpu.create_texture(&logo_image);
        let logo_mesh = gpu.create_mesh(&LOGO_VERTICES, &LOGO_INDICES);
        let logo = gui.image(logo_texture, logo_mesh);

        TitleGui {
            gui: gui_renderer.build_gui(gpu, gui),
            gui_resources: gui_renderer,
            singleplayer,
            multiplayer,
            options,
            quit,
            logo,
        }
    }

    pub fn handle_resize(&mut self, gpu: &GpuWrapper, width: f32, height: f32) {
        let logo = self.gui.image(self.logo);
        logo.height = height * 0.15;
        logo.width = logo.height * LOGO_ASPECT;
        logo.set_pos(width / 2. - logo.width / 2., height * 0.9 - logo.height);

        self.gui.update_button_scales(height);

        let singleplayer = self.gui.button(self.singleplayer);
        let x = width / 2. - singleplayer.width() / 2.;

        singleplayer.set_pos(x, height * 0.4);
        self.gui.button(self.multiplayer).set_pos(x, height * 0.28);
        self.gui.button(self.options).set_pos(x, height * 0.16);
        self.gui.button(self.quit).set_pos(x, height * 0.04);
        self.gui.resize(gpu);
    }

    pub fn handle_mouse_move(&mut self, gpu: &GpuWrapper, position: PhysicalPosition<f32>) {
        self.gui.mouse_moved(gpu, position);
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.gui.render(render_pass, &self.gui_resources);
    }
}
