use std::rc::Rc;

use wgpu::RenderPass;
use winit::dpi::PhysicalPosition;

use crate::gpu::GpuWrapper;

use super::{GuiResources, GuiSpec, Gui, render::SpriteVertex};

const BACKGROUND_VERTICES: [SpriteVertex; 4] = [
    SpriteVertex {
        pos: [0., 0.],
        uv: [0., 1.],
    },
    SpriteVertex {
        pos: [1., 0.],
        uv: [1., 1.],
    },

    SpriteVertex {
        pos: [1., 1.],
        uv: [1., 0.],
    },
    SpriteVertex {
        pos: [0., 1.],
        uv: [0., 0.],
    },
];
const BACKGROUND_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

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
    background: usize,
    background_aspect: f32,
    logo: usize,
}

impl TitleGui {
    pub fn new(gpu: &GpuWrapper, gui_renderer: Rc<GuiResources>) -> TitleGui {
        let mut gui = GuiSpec::new();
        let singleplayer = gui.button("Singleplayer");
        let multiplayer = gui.button("Multiplayer");
        let options = gui.button("Options");
        let quit = gui.button("Quit");

        let background_image = image::load_from_memory(include_bytes!("../../res/background.png")).unwrap();
        let background_texture = gpu.create_texture(&background_image);
        let background_mesh = gpu.create_mesh(&BACKGROUND_VERTICES, &BACKGROUND_INDICES);
        let background = gui.image(background_texture, background_mesh);

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
            background,
            background_aspect: background_image.width() as f32 / background_image.height() as f32,
            logo,
        }
    }

    pub fn handle_resize(&mut self, gpu: &GpuWrapper, width: f32, height: f32) {
        let background = self.gui.image(self.background);
        if width / height > self.background_aspect {
            background.width = width;
            background.height = width / self.background_aspect;
        } else {
            background.width = height * self.background_aspect;
            background.height = height;
            background.set_pos(width / 2. - background.width / 2., height / 2. - background.height / 2.);
        }
        background.set_pos(width / 2. - background.width / 2., height / 2. - background.height / 2.);

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
