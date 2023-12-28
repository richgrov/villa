use wgpu::{util::DeviceExt, BufferUsages, IndexFormat};

use crate::gpu::GpuWrapper;

use super::{render::FontVertex, GuiResources};

pub struct BakedText {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    num_indices: u32,
    width: f32,
}

impl<'a> BakedText {
    pub fn new(
        gpu: &GpuWrapper,
        resources: &GuiResources,
        text: &str,
        shadow: bool,
    ) -> Option<Self> {
        let mesh_factor = if shadow { 2 } else { 1 };
        let mut vertices = Vec::with_capacity(text.len() * 4 * mesh_factor);
        let mut indices = Vec::with_capacity(text.len() * 6 * mesh_factor);
        let mut x_offset = 0.;
        let mut index_offset = 0u32;

        let chars: Vec<_> = text.chars().enumerate().collect();
        for (i, c) in &chars {
            // Minecraft ignores the first 2 rows of characters so add 32 to the index
            let index = resources.font_map.iter().position(|ch| *ch == *c)? + 32;
            let uv = &resources.character_uv[index];
            let render_width = uv.width() * 16.;

            let vertex_data = [
                FontVertex {
                    pos: [x_offset, 0.],
                    uv: [uv.left(), uv.top() + 1. / 16.],
                    color: [1.0, 1.0, 1.0],
                },
                FontVertex {
                    pos: [x_offset + render_width, 0.],
                    uv: [uv.left() + uv.width(), uv.top() + 1. / 16.],
                    color: [1.0, 1.0, 1.0],
                },
                FontVertex {
                    pos: [x_offset + render_width, 1.],
                    uv: [uv.left() + uv.width(), uv.top()],
                    color: [1.0, 1.0, 1.0],
                },
                FontVertex {
                    pos: [x_offset, 1.],
                    uv: [uv.left(), uv.top()],
                    color: [1.0, 1.0, 1.0],
                },
            ];

            if shadow {
                let mut shadow_text = vertex_data.clone();
                for v in &mut shadow_text {
                    v.pos[0] += 1. / 16.;
                    v.pos[1] -= 1. / 16.;
                    v.color = [0.24705882352, 0.24705882352, 0.24705882352];
                }
                vertices.extend_from_slice(&shadow_text);
            }

            vertices.extend_from_slice(&vertex_data);

            if *i < chars.len() - 1 {
                x_offset += render_width + 1. / 16. /* one pixel space between letters */;
            } else {
                x_offset += render_width;
            }

            for _ in 0..mesh_factor {
                indices.extend_from_slice(&[
                    index_offset,
                    index_offset + 1,
                    index_offset + 2,
                    index_offset + 2,
                    index_offset + 3,
                    index_offset,
                ]);
                index_offset += 4;
            }
        }

        let vertex_buf = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Baked text vertices: {}", text)),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });

        let index_buf = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Baked text indices: {}", text)),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX,
            });

        Some(BakedText {
            vertex_buf,
            index_buf,
            num_indices: indices.len() as u32,
            width: x_offset,
        })
    }

    pub fn bind(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        render_pass.set_index_buffer(self.index_buf.slice(..), IndexFormat::Uint32);
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}
