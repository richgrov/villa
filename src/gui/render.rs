use glam::{Mat4, Vec3};
use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::gpu::{GpuWrapper, Mesh};

pub struct GuiSpec {
    buttons: Vec<(f32, f32, String)>,
}

impl GuiSpec {
    pub fn new() -> GuiSpec {
        GuiSpec {
            buttons: Vec::with_capacity(2),
        }
    }

    pub fn button(mut self, x: f32, y: f32, text: &str) -> Self {
        self.buttons.push((x, y, text.to_owned()));
        self
    }
}

pub struct GuiRenderer {
    font_pipeline: wgpu::RenderPipeline,
    font_texture: wgpu::BindGroup,
    font_uniforms_layout: wgpu::BindGroupLayout,
    font_uniform_size: usize,
    font_map: Vec<char>,
    // Left, top, width
    glyph_coords: Vec<(f32, f32, f32)>,
}

impl GuiRenderer {
    pub fn new(gpu: &GpuWrapper) -> GuiRenderer {
        let uniform_size = wgpu::util::align_to(
            std::mem::size_of::<FontMeshUniforms>(),
            gpu.device().limits().min_uniform_buffer_offset_alignment as usize
        );

        let texture_layout = gpu.create_bind_group_layout("Font Texture", &[
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
             },
        ]);

        let uniforms_layout = gpu.create_bind_group_layout("Font Uniforms", &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(uniform_size as u64),
            },
            count: None,
        }]);

        let pipeline = gpu.create_pipeline::<Vertex>("Font", include_str!("../../res/font.wgsl"), &[&texture_layout, &uniforms_layout]);

        let image = image::load_from_memory(include_bytes!("../../res/default.png")).unwrap();
        if image.width() % 16 != 0 {
            panic!("font texture should have 16 evenly-sized columns");
        }
        if image.height() % 16 != 0 {
            panic!("font texture should have 16 evenly-sized rows");
        }

        let cell_width = image.width() / 16;
        let cell_height = image.height() / 16;
        let widthf = image.width() as f32;
        let mut glyph_coords = vec![(0f32, 0f32, 0f32); 256];
        let font_map: Vec<_> = include_str!("../../res/font.txt").to_owned().replace("\n", "").chars().collect();
        for x in 0..image.width() {
            for y in 0..image.height() {
                let current_cell_x = x / cell_width;
                let current_cell_y = y / cell_height;
                if image.get_pixel(x, y)[3] != 0 {
                    let size = &mut glyph_coords[(current_cell_y * 16 + current_cell_x) as usize];
                    size.2 = size.2.max(((x + 1) % cell_width) as f32 / widthf);
                }
            }
        }
        for x in 0..16 {
            for y in 0..16 {
                let size = &mut glyph_coords[(y * 16 + x) as usize];
                size.0 = x as f32 / 16.;
                size.1 = y as f32 / 16.;
            }
        }

        let texture_bind_group = gpu.create_texture(&image, &texture_layout);

        if let Some(index) = font_map.iter().position(|c| *c == ' ') {
            glyph_coords[index + 32].2 = 4. / 256.;
        }

        GuiRenderer {
            font_pipeline: pipeline,
            font_texture: texture_bind_group,
            font_uniforms_layout: uniforms_layout,
            font_uniform_size: uniform_size,

            font_map,
            glyph_coords,
        }
    }

    pub fn build_text(&self, gpu: &GpuWrapper, text: &str, shadow: bool) -> Option<Mesh> {
        let mesh_factor = if shadow { 2 } else { 1 };
        let mut vertices = Vec::with_capacity(text.len() * 4 * mesh_factor);
        let mut indices = Vec::with_capacity(text.len() * 6 * mesh_factor);
        let mut x_offset = 0.;
        let mut index_offset = 0u32;

        for c in text.chars() {
            // Minecraft ignores the first 2 rows of characters so add 32 to the index
            let index = self.font_map.iter().position(|ch| *ch == c)? + 32;
            let (left, top, width) = self.glyph_coords[index];
            let render_width = width * 16.;

            let vertex_data = [
                Vertex {
                    pos: [x_offset, 0.],
                    uv: [left, top + 1./16.],
                    color: [1.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [x_offset + render_width, 0.],
                    uv: [left + width, top + 1./16.],
                    color: [1.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [x_offset + render_width, 1.],
                    uv: [left + width, top],
                    color: [1.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [x_offset, 1.],
                    uv: [left, top],
                    color: [1.0, 1.0, 1.0],
                },
            ];

            if shadow {
                let mut shadow_text = vertex_data.clone();
                for v in &mut shadow_text {
                    v.pos[0] += 1./16.;
                    v.pos[1] -= 1./16.;
                    v.color = [0.24705882352, 0.24705882352, 0.24705882352];
                }
                vertices.extend_from_slice(&shadow_text);
            }

            vertices.extend_from_slice(&vertex_data);
            x_offset += render_width + 1. / 16. /* one pixel space between letters */;

            for _ in 0..mesh_factor {
                indices.extend_from_slice(&[
                    index_offset, index_offset + 1, index_offset + 2, index_offset + 2, index_offset + 3, index_offset
                ]);
                index_offset += 4;
            }
        }

        Some(gpu.create_mesh(&vertices, &indices))
    }

    pub fn build_gui(&self, gpu: &GpuWrapper, gui_spec: &GuiSpec) -> Gui {
        let size = gpu.window_size();

        let projection = Mat4::orthographic_lh(
            0.,
            size.width as f32,
            0.,
            size.height as f32,
            -1.,
            1.,
        );

        let mut buffer = vec![0u8; gui_spec.buttons.len() * self.font_uniform_size];
        let mut buttons = Vec::with_capacity(gui_spec.buttons.len());
        for (x, y, text) in &gui_spec.buttons {
            let button = Button {
                x: *x,
                y: *y,
                baked_text: self.build_text(&gpu, text, true).unwrap(),
                primary_uniform_offset: (buttons.len() * self.font_uniform_size) as u32,
            };

            let uniform = button.build_uniform(&projection, size.width as f32, size.height as f32);
            let uniform_buf = bytemuck::bytes_of(&uniform);
            let offset = button.primary_uniform_offset as usize;
            buffer[offset..offset+std::mem::size_of::<FontMeshUniforms>()].copy_from_slice(uniform_buf);
            buttons.push(button);
        }

        let uniforms_buffer = gpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font uniforms buffer"),
            contents: bytemuck::cast_slice(&buffer),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.font_uniforms_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniforms_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(self.font_uniform_size as u64),
                }),
            }],
            label: None,
        });

        Gui {
            buttons,
            button_uniforms: uniforms_buffer,
            uniforms_bind_group: bind_group,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, gui: &'a Gui) {
        render_pass.set_pipeline(&self.font_pipeline);
        render_pass.set_bind_group(0, &self.font_texture, &[]);
        for button in &gui.buttons {
            render_pass.set_bind_group(1, &gui.uniforms_bind_group, &[button.primary_uniform_offset]);
            button.baked_text.bind(render_pass);
            button.baked_text.draw(render_pass);
        }
    }
}

struct Button {
    x: f32,
    y: f32,
    baked_text: Mesh,
    primary_uniform_offset: wgpu::DynamicOffset,
}

impl Button {
    fn build_uniform(&self, projection: &Mat4, width: f32, height: f32) -> FontMeshUniforms {
        let model = Mat4::from_translation(Vec3::new(self.x * width, self.y * height, 0.)) * Mat4::from_scale(Vec3::new(100., 100., 1.));
        let mvp = *projection * model;

        FontMeshUniforms {
            mvp: mvp.to_cols_array(),
            color: [1.0, 1.0, 1.0],
        }
    }
}

pub struct Gui {
    buttons: Vec<Button>,
    button_uniforms: wgpu::Buffer,
    uniforms_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 3],
}

const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x3];

impl crate::gpu::VertexAttribues for Vertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &ATTRIBS
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FontMeshUniforms {
    pub mvp: [f32; 16],
    pub color: [f32; 3],
}
