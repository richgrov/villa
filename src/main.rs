mod graphics;

use glam::{Mat4, Vec3};
use image::GenericImageView;
use winit::{event_loop::{EventLoop, ControlFlow}, window::{Window, WindowBuilder}};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

impl graphics::VertexAttribues for Vertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &ATTRIBS
    }
}

pub struct App {
    window: Window,
    gpu: graphics::GpuWrapper,

    font_pipeline: wgpu::RenderPipeline,
    font_bind_group: wgpu::BindGroup,
    font_camera_buf: wgpu::Buffer,
    font_uniform: wgpu::BindGroup,
    font_mesh: Option<graphics::Mesh>,

    font_map: Vec<char>,
    // Left, top, width
    glyph_coords: Vec<(f32, f32, f32)>,
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

        let gpu = graphics::GpuWrapper::new(&window).await;

        let texture_bind_layout = gpu.create_bind_group_layout(&[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
        ]);
        let camera_bind_layout = gpu.create_bind_group_layout(&[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]);

        let font_pipeline = gpu.create_pipeline::<Vertex>(include_str!("../res/font.wgsl"), &[&texture_bind_layout, &camera_bind_layout]);

        let image = image::load_from_memory(include_bytes!("../res/default.png")).unwrap();
        if image.width() % 16 != 0 {
            panic!("font texture should have 16 evenly-sized columns");
        }
        if image.height() % 16 != 0 {
            panic!("font texture should have 16 evenly-sized rows");
        }

        let cell_width = image.width() / 16;
        let cell_height = image.height() / 16;
        let widthf = image.width() as f32;
        let mut glyph_sizes = vec![(0f32, 0f32, 0f32); 256];
        for x in 0..image.width() {
            for y in 0..image.height() {
                let current_cell_x = x / cell_width;
                let current_cell_y = y / cell_height;
                if image.get_pixel(x, y)[3] != 0 {
                    let size = &mut glyph_sizes[(current_cell_y * 16 + current_cell_x) as usize];
                    size.2 = size.2.max(((x + 1) % cell_width) as f32 / widthf);
                }
            }
        }
        for x in 0..16 {
            for y in 0..16 {
                let size = &mut glyph_sizes[(y * 16 + x) as usize];
                size.0 = x as f32 / 16.;
                size.1 = y as f32 / 16.;
            }
        }

        let texture = gpu.create_texture(&image, &texture_bind_layout);

        let (camera_buf, camera_uniform) = gpu.create_uniform(&[
            1.0f32, 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ], &camera_bind_layout);

        let mut app = App {
            window,
            gpu,

            font_pipeline,
            font_bind_group: texture,
            font_camera_buf: camera_buf,
            font_uniform: camera_uniform,
            font_mesh: None,

            font_map: include_str!("../res/font.txt").to_owned().replace("\n", "").chars().collect(),
            glyph_coords: glyph_sizes,
        };
        
        if let Some(index) = app.font_map.iter().position(|c| *c == ' ') {
            app.glyph_coords[index + 32].2 = 4. / 256.;
        }
        app
    }

    fn build_text(&self, text: &str) -> Option<graphics::Mesh> {
        let mut vertices = Vec::with_capacity(text.len() * 4);
        let mut indices = Vec::with_capacity(text.len() * 6);
        let mut x_offset = 0.;
        let mut index_offset = 0u32;

        for c in text.chars() {
            // Minecraft ignores the first 2 rows of characters so add 32 to the index
            let index = self.font_map.iter().position(|ch| *ch == c)? + 32;
            let (left, top, width) = self.glyph_coords[index];
            let height = 1./16.;

            vertices.extend_from_slice(&[
                Vertex { pos: [x_offset, 0.], uv: [left, top + height] },
                Vertex { pos: [x_offset + width, 0.], uv: [left + width, top + height] },
                Vertex { pos: [x_offset + width, height], uv: [left + width, top] },
                Vertex { pos: [x_offset, height], uv: [left, top] },
            ]);
            x_offset += width + 2. / 256.;

            indices.extend_from_slice(&[
                index_offset, index_offset + 1, index_offset + 2, index_offset + 2, index_offset + 3, index_offset
            ]);
            index_offset += 4;
        }

        Some(self.gpu.create_mesh(&vertices, &indices))
    }

    fn update(&mut self) {
        if self.font_mesh.is_none() {
            self.font_mesh = self.build_text("This is some test Text!");
        }
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

            if let Some(mesh) = &self.font_mesh {
                let (width, height) = (1280., 720.); // TODO
                let projection = Mat4::orthographic_lh(0., width, 0., height, -1., 1.);
                let view = Mat4::IDENTITY;
                let model = Mat4::from_scale(Vec3::new(700., 700., 700.));
                let mvp = projection * view * model;
                self.gpu.update_buffer(&self.font_camera_buf, &mvp.to_cols_array());

                pass.set_pipeline(&self.font_pipeline);
                pass.set_bind_group(0, &self.font_bind_group, &[]);
                pass.set_bind_group(1, &self.font_uniform, &[]);
                mesh.bind(&mut pass);
                mesh.draw(&mut pass);
            }
        }

        self.gpu.queue_commands(encoder.finish());
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
                    WindowEvent::Resized(size) => app.gpu.handle_resize(size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => app.gpu.handle_resize(*new_inner_size),
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

