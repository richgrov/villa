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

    button_pipeline: wgpu::RenderPipeline,
    gui_texture: wgpu::BindGroup,
    gui_button_large: Mesh,
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

        let pipeline = gpu.create_pipeline::<FontVertex>("Font", include_str!("../../res/font.wgsl"), &[&texture_layout, &uniforms_layout]);

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

        let gui_texture_pipeline = gpu.create_pipeline::<GuiTextureVertex>("Gui Texture", include_str!("../../res/gui.wgsl"), &[&texture_layout, &uniforms_layout]);
        let gui_image = image::load_from_memory(include_bytes!("../../res/gui.png")).unwrap();
        let gui_texture = gpu.create_texture(&gui_image, &texture_layout);
        let gui_button_large = gpu.create_mesh(&[
            GuiTextureVertex {
                pos: [0., 0.],
                uv: [0., 65./256.],
            },
            GuiTextureVertex {
                pos: [1., 0.],
                uv: [199./256., 65./256.],
            },
            GuiTextureVertex {
                pos: [1., 1.],
                uv: [199./256., 46./256.],
            },
            GuiTextureVertex {
                pos: [0., 1.],
                uv: [0., 46./256.],
            },
        ], &[0u16, 1, 2, 2, 3, 0]);

        GuiRenderer {
            font_pipeline: pipeline,
            font_texture: texture_bind_group,
            font_uniforms_layout: uniforms_layout,
            font_uniform_size: uniform_size,
            font_map,
            glyph_coords,

            button_pipeline: gui_texture_pipeline,
            gui_texture,
            gui_button_large,
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
                FontVertex {
                    pos: [x_offset, 0.],
                    uv: [left, top + 1./16.],
                    color: [1.0, 1.0, 1.0],
                },
                FontVertex {
                    pos: [x_offset + render_width, 0.],
                    uv: [left + width, top + 1./16.],
                    color: [1.0, 1.0, 1.0],
                },
                FontVertex {
                    pos: [x_offset + render_width, 1.],
                    uv: [left + width, top],
                    color: [1.0, 1.0, 1.0],
                },
                FontVertex {
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

        let mut buffer = vec![0u8; gui_spec.buttons.len() * self.font_uniform_size * 2];
        let mut buttons = Vec::with_capacity(gui_spec.buttons.len());
        for (x, y, text) in &gui_spec.buttons {
            let button = Button {
                x: *x,
                y: *y,
                baked_text: self.build_text(&gpu, text, true).unwrap(),
                primary_uniform_offset: (buttons.len() * self.font_uniform_size) as u32,
            };

            let background_offset = button.primary_uniform_offset as usize;
            let background_uniform = button.build_background_uniform(&projection, size.width as f32, size.height as f32);
            let background_uniform_buf = bytemuck::bytes_of(&background_uniform);
            buffer[background_offset..background_offset+std::mem::size_of::<FontMeshUniforms>()].copy_from_slice(background_uniform_buf);

            let text_offset = button.primary_uniform_offset as usize + gui_spec.buttons.len() * self.font_uniform_size;
            let text_uniform = button.build_text_uniform(&projection, size.width as f32, size.height as f32);
            let text_uniform_buf = bytemuck::bytes_of(&text_uniform);
            buffer[text_offset..text_offset+std::mem::size_of::<FontMeshUniforms>()].copy_from_slice(text_uniform_buf);

            buttons.push(button);
        }

        let button_uniforms = gpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font uniforms buffer"),
            contents: bytemuck::cast_slice(&buffer),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let button_background_bind = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.font_uniforms_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &button_uniforms,
                    offset: 0,
                    size: wgpu::BufferSize::new(self.font_uniform_size as u64),
                }),
            }],
            label: Some("Button background binding"),
        });

        let button_text_bind = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.font_uniforms_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &button_uniforms,
                    offset: (gui_spec.buttons.len() * self.font_uniform_size) as u64,
                    size: wgpu::BufferSize::new(self.font_uniform_size as u64),
                }),
            }],
            label: Some("Button text binding"),
        });

        Gui {
            buttons,
            button_uniforms,
            button_background_bind,
            button_text_bind,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, gui: &'a Gui) {
        render_pass.set_pipeline(&self.button_pipeline);
        render_pass.set_bind_group(0, &self.gui_texture, &[]);
        self.gui_button_large.bind(render_pass);
        for button in &gui.buttons {
            render_pass.set_bind_group(1, &gui.button_background_bind, &[button.primary_uniform_offset]);
            self.gui_button_large.draw(render_pass);
        }

        render_pass.set_pipeline(&self.font_pipeline);
        render_pass.set_bind_group(0, &self.font_texture, &[]);
        for button in &gui.buttons {
            render_pass.set_bind_group(1, &gui.button_text_bind, &[button.primary_uniform_offset]);
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
    fn build_background_uniform(&self, projection: &Mat4, width: f32, height: f32) -> FontMeshUniforms {
        let model = Mat4::from_translation(Vec3::new(self.x * width, self.y * height, 0.)) * Mat4::from_scale(Vec3::new(500., 100., 1.));
        let mvp = *projection * model;

        FontMeshUniforms {
            mvp: mvp.to_cols_array(),
            color: [1.0, 1.0, 1.0],
        }
    }

    fn build_text_uniform(&self, projection: &Mat4, width: f32, height: f32) -> FontMeshUniforms {
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
    button_background_bind: wgpu::BindGroup,
    button_text_bind: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FontVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 3],
}

const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x3];

impl crate::gpu::VertexAttribues for FontVertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &ATTRIBS
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GuiTextureVertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

const GUI_VERTEX_ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

impl crate::gpu::VertexAttribues for GuiTextureVertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &GUI_VERTEX_ATTRIBS
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FontMeshUniforms {
    pub mvp: [f32; 16],
    pub color: [f32; 3],
}
