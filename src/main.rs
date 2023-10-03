mod gui;
mod render;

use glam::{Mat4, Vec3};
use gui::{TitleGui, Gui};
use winit::{event_loop::{EventLoop, ControlFlow}, window::{Window, WindowBuilder}, dpi::PhysicalSize};

pub struct App {
    window: Window,
    window_size: PhysicalSize<u32>,
    renderer: render::Renderer,

    gui: TitleGui,
}

impl App {
    pub async fn new(event_loop: &EventLoop<()>) -> App {
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize {
                width: 1280,
                height: 720,
            })
            .with_title("golden")
            .build(&event_loop)
            .unwrap();
        let window_size = window.inner_size();

        let renderer = render::Renderer::new(&window).await;
        let title = TitleGui::new(&renderer);

        App {
            window,
            window_size,
            renderer,
            gui: title,
        }
    }

    fn update(&mut self) {
    }

    fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;
        self.renderer.gpu_mut().handle_resize(new_size);
    }

    fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let (frame, view, mut encoder) = self.renderer.gpu().begin_draw()?;

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

            self.draw_gui(&mut pass);
        }

        self.renderer.gpu_mut().queue_commands(encoder.finish());
        frame.present();

        Ok(())
    }

    fn draw_gui<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>) {
        let buttons = self.gui.buttons();
        if buttons.is_empty() {
            return
        }

        self.renderer.font().prepare(pass);

        let (width, height) = (self.window_size.width as f32, self.window_size.height as f32);

        let projection = Mat4::orthographic_lh(
            0.,
            width,
            0.,
            height,
            -1.,
            1.,
        );

        for button in buttons {
            let model = Mat4::from_translation(Vec3::new(button.x() * width, button.y() * height, 0.)) * Mat4::from_scale(Vec3::new(100., 100., 1.));
            let mvp = projection * model;
            self.renderer.font().set_camera(self.renderer.gpu(), &mvp);

            button.mesh().bind(pass);
            button.mesh().draw(pass);
        }
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
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => app.renderer.gpu().reconfigure_surface(),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                        _ => {},
                    }
                },
                Event::WindowEvent { window_id, event } if window_id == app.window.id() => match event {
                    WindowEvent::Resized(size) => app.handle_resize(size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => app.handle_resize(*new_inner_size),
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
