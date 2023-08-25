use image::GenericImageView;

use super::gpu::GpuWrapper;

pub struct FontRenderer {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    camera_buf: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    font_map: Vec<char>,
    // Left, top, width
    glyph_coords: Vec<(f32, f32, f32)>,
}

impl FontRenderer {
    pub fn new(gpu: &GpuWrapper) -> FontRenderer {
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

        let camera_layout = gpu.create_bind_group_layout("Font Camera", &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]);

        let pipeline = gpu.create_pipeline::<Vertex>("Font", include_str!("../../res/font.wgsl"), &[&texture_layout, &camera_layout]);

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

        let (camera_buf, camera_bind_group) = gpu.create_uniform(&[
            1.0f32, 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ], &camera_layout);

        if let Some(index) = font_map.iter().position(|c| *c == ' ') {
            glyph_coords[index + 32].2 = 4. / 256.;
        }

        FontRenderer {
            pipeline,
            texture_bind_group,
            camera_buf,
            camera_bind_group,

            font_map,
            glyph_coords,
        }
    }

    pub fn build_text(&self, gpu: &GpuWrapper, text: &str) -> Option<super::Mesh> {
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

        Some(gpu.create_mesh(&vertices, &indices))
    }

    pub fn prepare<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.texture_bind_group, &[]);
        pass.set_bind_group(1, &self.camera_bind_group, &[]);
    }

    pub fn set_camera(&self, gpu: &GpuWrapper, mvp: &glam::Mat4) {
        gpu.update_buffer(&self.camera_buf, &mvp.to_cols_array());
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

impl super::gpu::VertexAttribues for Vertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &ATTRIBS
    }
}
