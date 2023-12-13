mod gui;
mod gpu;
mod scene;
mod uniforms;
mod world;

use std::rc::Rc;

use gpu::GpuWrapper;
use gui::TitleGui;
use scene::{Scene, NextState};
use winit::{event_loop::{EventLoop, ControlFlow}, window::{Window, WindowBuilder}, dpi::{PhysicalSize, PhysicalPosition}, event::{ElementState, MouseButton}};

pub struct App {
    window: Window,
    window_size: PhysicalSize<u32>,
    gpu: gpu::GpuWrapper,

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
        title.handle_resize(&gpu, window_size.width as f32, window_size.height as f32);

        App {
            window,
            window_size,
            gpu,
            current_scene: Box::new(title),
        }
    }

    fn update(&mut self) {
    }

    fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;
        self.gpu.handle_resize(new_size);
        self.current_scene.handle_resize(&self.gpu, new_size.width as f32, new_size.height as f32);
    }

    fn mouse_moved(&mut self, position: PhysicalPosition<f64>) {
        let converted = PhysicalPosition::new(
            position.x as f32,
            self.window_size.height as f32 - position.y as f32
        );
        self.current_scene.handle_mouse_move(&self.gpu, converted);
    }

    fn handle_click(&mut self, state: ElementState, button: MouseButton) -> bool {
        let next_state = self.current_scene.handle_click(&self.gpu, state, button);
        match next_state {
            NextState::Continue => {},
            NextState::ChangeScene(scene) => self.set_scene(scene),
            NextState::Exit => return true,
        }

        false
    }

    fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let (frame, view, mut encoder) = self.gpu.begin_draw()?;

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.current_scene.draw(&mut pass);
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

fn main() {
    env_logger::init();

    pollster::block_on(async move {
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

                    app.update();

                    match app.draw() {
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => app.gpu.reconfigure_surface(),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                        _ => {},
                    }
                },
                Event::WindowEvent { window_id, event } if window_id == app.window.id() => match event {
                    WindowEvent::Resized(size) => app.handle_resize(size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => app.handle_resize(*new_inner_size),
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::CursorMoved { position, .. } => app.mouse_moved(position),
                    WindowEvent::MouseInput { state, button, .. } => {
                        let should_exit = app.handle_click(state, button);
                        if should_exit {
                            *control_flow = ControlFlow::Exit;
                        }
                    },
                    _ => {},
                },
                Event::MainEventsCleared => {
                    app.window.request_redraw();
                }
                _ => {},
            }
        });
    });
}
