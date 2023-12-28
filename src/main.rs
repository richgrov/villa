mod gpu;
mod gui;
mod net;
mod scene;
mod uniforms;
mod world;

use std::rc::Rc;

use gpu::GpuWrapper;
use gui::TitleGui;
use scene::{NextState, Scene};
use wgpu::StoreOp;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyboardInput, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct App {
    window: Window,
    window_size: PhysicalSize<u32>,
    gpu: gpu::GpuWrapper,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,

    current_scene: Box<dyn Scene>,
}

impl App {
    pub async fn new(event_loop: &EventLoop<()>) -> App {
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize {
                width: 1280,
                height: 720,
            })
            .with_title("villa")
            .build(&event_loop)
            .unwrap();

        let gpu = GpuWrapper::new(&window).await;
        let gui_resources = Rc::new(gui::GuiResources::new(&gpu));
        let mut title = TitleGui::new(&gpu, gui_resources.clone());

        let window_size = window.inner_size();
        let (depth_texture, depth_texture_view) =
            gpu.create_depth_texture(window_size.width, window_size.height);
        title.handle_resize(&gpu, window_size.width as f32, window_size.height as f32);

        App {
            window,
            window_size,
            gpu,
            depth_texture,
            depth_texture_view,

            current_scene: Box::new(title),
        }
    }

    /// Returns `true` if the app should exit
    fn update(&mut self) -> bool {
        match self.current_scene.update(&self.gpu) {
            NextState::Continue => {}
            NextState::ChangeScene(scene) => self.set_scene(scene),
            NextState::Exit => return true,
        }

        false
    }

    fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;
        self.gpu.handle_resize(new_size);
        let (depth_texture, depth_texture_view) = self
            .gpu
            .create_depth_texture(new_size.width, new_size.height);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
        self.current_scene
            .handle_resize(&self.gpu, new_size.width as f32, new_size.height as f32);
    }

    fn mouse_moved(&mut self, position: PhysicalPosition<f64>) {
        let converted = PhysicalPosition::new(
            position.x as f32,
            self.window_size.height as f32 - position.y as f32,
        );
        self.current_scene.handle_mouse_move(&self.gpu, converted);
    }

    /// Returns `true` if the app should exit
    fn handle_click(&mut self, state: ElementState, button: MouseButton) -> bool {
        let next_state = self.current_scene.handle_click(&self.gpu, state, button);
        match next_state {
            NextState::Continue => {}
            NextState::ChangeScene(scene) => self.set_scene(scene),
            NextState::Exit => return true,
        }

        false
    }

    fn handle_key_input(&mut self, key: KeyboardInput) {
        self.current_scene.handle_key_input(&self.gpu, key);
    }

    fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let (frame, view, mut encoder) = self.gpu.begin_draw()?;

        {
            let mut d3_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.6,
                            g: 0.75,
                            b: 1.,
                            a: 1.,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            self.current_scene.draw_3d(&mut d3_pass);
        }
        {
            let mut ui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            self.current_scene.draw_ui(&mut ui_pass);
        }
        self.gpu.queue().submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    fn set_scene(&mut self, scene: Box<dyn Scene>) {
        self.current_scene = scene;
        self.current_scene.handle_resize(
            &self.gpu,
            self.window_size.width as f32,
            self.window_size.height as f32,
        );
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let mut app = App::new(&event_loop).await;

    let mut time = std::time::Instant::now();
    let mut frames = 0;

    event_loop.run(move |event, _, control_flow| {
        use winit::event::{Event, WindowEvent};
        match event {
            Event::RedrawRequested(_) => {
                frames += 1;
                let now = std::time::Instant::now();
                if now - time > std::time::Duration::from_secs(1) {
                    time = now;
                    println!("{}", frames);
                    frames = 0;
                }

                let should_exit = app.update();
                if should_exit {
                    *control_flow = ControlFlow::Exit;
                }

                match app.draw() {
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        app.gpu.reconfigure_surface()
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                    _ => {}
                }
            }
            Event::WindowEvent { window_id, event } if window_id == app.window.id() => {
                match event {
                    WindowEvent::Resized(size) => app.handle_resize(size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        app.handle_resize(*new_inner_size)
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::CursorMoved { position, .. } => app.mouse_moved(position),
                    WindowEvent::MouseInput { state, button, .. } => {
                        let should_exit = app.handle_click(state, button);
                        if should_exit {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    WindowEvent::KeyboardInput { input, .. } => app.handle_key_input(input),
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                app.window.request_redraw();
            }
            _ => {}
        }
    });
}
