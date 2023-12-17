use std::rc::Rc;

use tokio::sync::mpsc;
use wgpu::RenderPass;
use winit::{dpi::PhysicalPosition, event::{ElementState, MouseButton}};

use crate::{gpu::GpuWrapper, scene::{Scene, NextState}, world::{World, WorldResources}, net::Connection};

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
    gui: Gui,
    gui_resources: Rc<GuiResources>,
    last_mouse_pos: PhysicalPosition<f32>,
    play: usize,
    options: usize,
    quit: usize,
    background: usize,
    background_aspect: f32,
    logo: usize,

    connection_tx: mpsc::Sender<Result<Connection, std::io::Error>>,
    connection_rx: mpsc::Receiver<Result<Connection, std::io::Error>>,
}

impl TitleGui {
    pub fn new(gpu: &GpuWrapper, gui_renderer: Rc<GuiResources>) -> TitleGui {
        let mut gui = GuiSpec::new();
        let play = gui.button("Play");
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

        let (tx, rx) = mpsc::channel(1);

        TitleGui {
            gui: gui_renderer.build_gui(gpu, gui),
            gui_resources: gui_renderer,
            last_mouse_pos: PhysicalPosition { x: 0., y: 0. },
            play,
            options,
            quit,
            background,
            background_aspect: background_image.width() as f32 / background_image.height() as f32,
            logo,

            connection_tx: tx,
            connection_rx: rx,
        }
    }
}

impl Scene for TitleGui {
    fn handle_resize(&mut self, gpu: &GpuWrapper, width: f32, height: f32) {
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

        let play = self.gui.button(self.play);
        let x = width / 2. - play.width() / 2.;
        play.set_pos(x, height * 0.3);
        self.gui.button(self.options).set_pos(x, height * 0.2);
        self.gui.button(self.quit).set_pos(x, height * 0.1);
        self.gui.resize(gpu);
    }

    fn handle_mouse_move(&mut self, gpu: &GpuWrapper, position: PhysicalPosition<f32>) {
        self.last_mouse_pos = position;
        self.gui.mouse_moved(gpu, position);
    }

    fn handle_click(&mut self, gpu: &GpuWrapper, state: ElementState, button: MouseButton) -> NextState {
        if button != MouseButton::Left {
            return NextState::Continue
        }

        if let Some(button_id) = self.gui.handle_click(gpu, state, self.last_mouse_pos) {
            if button_id == self.play {
                let tx = self.connection_tx.clone();
                tokio::spawn(async move {
                    let connection = Connection::connect("127.0.0.1:25565", "Steve").await;
                    if let Err(e) = tx.send(connection).await {
                        eprintln!("error sending connection through channel: {}", e);
                    }
                });
            } else if button_id == self.quit {
                return NextState::Exit
            }
        }

        NextState::Continue
    }

    fn handle_key_input(&mut self, _gpu: &GpuWrapper, _key: winit::event::KeyboardInput) {
    }

    fn update(&mut self, gpu: &GpuWrapper) -> NextState {
        match self.connection_rx.try_recv() {
            Ok(result) => match result {
                Ok(conn) => {
                    let resources = WorldResources::new(gpu);
                    return NextState::ChangeScene(Box::new(World::new(gpu, Rc::new(resources))))
                },
                Err(e) => eprintln!("failed to connect: {}", e),
            },
            _ => {},
        }

        NextState::Continue
    }

    fn draw_ui<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.gui.render(render_pass, &self.gui_resources);
    }
}
