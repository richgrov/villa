use image::GenericImageView;

use super::gpu::GpuWrapper;

pub struct FontRenderer {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    uniforms_layout: wgpu::BindGroupLayout,
    uniform_size: usize,

    font_map: Vec<char>,
    // Left, top, width
    glyph_coords: Vec<(f32, f32, f32)>,
}

impl FontRenderer {
    pub fn new(gpu: &GpuWrapper) -> FontRenderer {
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

        FontRenderer {
            pipeline,
            texture_bind_group,
            uniforms_layout,
            uniform_size,

            font_map,
            glyph_coords,
        }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn texture_bind_group(&self) -> &wgpu::BindGroup {
        &self.texture_bind_group
    }

    pub fn uniforms_layout(&self) -> &wgpu::BindGroupLayout {
        &self.uniforms_layout
    }

    pub fn uniform_size(&self) -> usize {
        self.uniform_size
    }

    pub fn build_text(&self, gpu: &GpuWrapper, text: &str, shadow: bool) -> Option<super::Mesh> {
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
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 3],
}

const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x3];

impl super::gpu::VertexAttribues for Vertex {
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
