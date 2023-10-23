use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::render::{self, FontMeshUniforms};

struct Button {
    x: f32,
    y: f32,
    text: String,
    baked_text: render::Mesh,
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

impl Gui {
    pub fn render<'a>(&'a self, renderer: &'a render::Renderer, pass: &mut wgpu::RenderPass<'a>) {
        self.render_strings(renderer, pass);
    }

    fn render_strings<'a>(&'a self, renderer: &'a render::Renderer, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(renderer.font().pipeline());
        pass.set_bind_group(0, renderer.font().texture_bind_group(), &[]);
        for button in &self.buttons {
            pass.set_bind_group(1, &self.uniforms_bind_group, &[button.primary_uniform_offset]);
            button.baked_text.bind(pass);
            button.baked_text.draw(pass);
        }
    }
}

pub struct GuiBuilder<'a> {
    renderer: &'a render::Renderer,
    buttons: Vec<Button>,
}

impl<'a> GuiBuilder<'a> {
    pub fn new(renderer: &render::Renderer) -> GuiBuilder {
        GuiBuilder {
            renderer,
            buttons: Vec::new(),
        }
    }

    pub fn button(mut self, x: f32, y: f32, text: &str) -> Self {
        let gpu = self.renderer.gpu();
        let button = Button {
            x,
            y,
            text: text.to_owned(),
            baked_text: self.renderer.font().build_text(&gpu, text, true).unwrap(),
            primary_uniform_offset: (self.buttons.len() * self.renderer.font().uniform_size()) as u32,
        };
        self.buttons.push(button);
        self
    }

    pub fn build(self) -> Gui {
        let size = self.renderer.gpu().window_size();

        let projection = Mat4::orthographic_lh(
            0.,
            size.width as f32,
            0.,
            size.height as f32,
            -1.,
            1.,
        );

        let mut buffer = vec![0u8; self.buttons.len() * self.renderer.font().uniform_size()];
        for button in &self.buttons {
            let uniform = button.build_uniform(&projection, size.width as f32, size.height as f32);
            let uniform_buf = bytemuck::bytes_of(&uniform);
            let offset = button.primary_uniform_offset as usize;
            buffer[offset..offset+std::mem::size_of::<FontMeshUniforms>()].copy_from_slice(uniform_buf);
        }

        let uniforms_buffer = self.renderer.gpu().device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font uniforms buffer"),
            contents: bytemuck::cast_slice(&buffer),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = self.renderer.gpu().device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: self.renderer.font().uniforms_layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniforms_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(self.renderer.font().uniform_size() as u64),
                }),
            }],
            label: None,
        });

        Gui {
            buttons: self.buttons,
            button_uniforms: uniforms_buffer,
            uniforms_bind_group: bind_group,
        }
    }
}
