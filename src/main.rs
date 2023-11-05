mod gui;
mod gpu;

use gpu::GpuWrapper;
use gui::TitleGui;
use winit::{event_loop::{EventLoop, ControlFlow}, window::{Window, WindowBuilder}, dpi::PhysicalSize};

pub struct App {
    window: Window,
    window_size: PhysicalSize<u32>,
    gpu: gpu::GpuWrapper,
    gui_renderer: gui::GuiRenderer,

    gui: TitleGui,
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
        let window_size = window.inner_size();

        let gpu = GpuWrapper::new(&window).await;
        let gui_renderer = gui::GuiRenderer::new(&gpu);
        let title = TitleGui::new(&gpu, &gui_renderer);

        App {
            window,
            window_size,
            gpu,
            gui_renderer,
            gui: title,
        }
    }

    fn update(&mut self) {
    }

    fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;
        self.gpu.handle_resize(new_size);
        self.gui_renderer.resize(&self.gpu, &self.gui.gui);
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

            self.gui_renderer.render(&mut pass, &self.gui.gui);
        }
        self.gpu.queue().submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
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
