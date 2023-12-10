use glam::{Mat4, Vec3};
use image::GenericImageView;
use winit::dpi::PhysicalPosition;

use crate::{gpu::{GpuWrapper, Mesh}, uniforms::{UniformStorage, UniformSpec}};

pub struct GuiSpec {
    buttons: Vec<String>,
}

impl GuiSpec {
    pub fn new() -> GuiSpec {
        GuiSpec {
            buttons: Vec::with_capacity(2),
        }
    }

    pub fn button(&mut self, text: &str) -> usize {
        let id = self.buttons.len();
        self.buttons.push(text.to_owned());
        id
    }
}

pub struct GuiResources {
    font_pipeline: wgpu::RenderPipeline,
    font_texture: wgpu::BindGroup,
    font_uniform_layout: UniformSpec,
    font_map: Vec<char>,
    // Left, top, width
    glyph_coords: Vec<(f32, f32, f32)>,

    button_pipeline: wgpu::RenderPipeline,
    gui_texture: wgpu::BindGroup,
    gui_button_large: Mesh,
    button_uniform_layout: UniformSpec,
}

impl GuiResources {
    pub fn new(gpu: &GpuWrapper) -> GuiResources {
        let texture_layout = gpu.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Font Texture"),
            entries: &[
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
            ],
        });

        let font_uniform_layout = UniformSpec::new::<Mat4>(gpu, "Font Uniform Layout", wgpu::ShaderStages::VERTEX);
        let button_uniform_layout = UniformSpec::new::<ButtonBackgroundUniform>(gpu, "Button Uniform", wgpu::ShaderStages::VERTEX);

        let pipeline = gpu.create_pipeline::<FontVertex>("Font", include_str!("../../res/font.wgsl"), &[&texture_layout, font_uniform_layout.layout()]);

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

        let gui_texture_pipeline = gpu.create_pipeline::<GuiTextureVertex>("Gui Texture", include_str!("../../res/gui.wgsl"), &[&texture_layout, button_uniform_layout.layout()]);
        let gui_image = image::load_from_memory(include_bytes!("../../res/gui.png")).unwrap();
        let gui_texture = gpu.create_texture(&gui_image, &texture_layout);
        let gui_button_large = gpu.create_mesh(&[
            GuiTextureVertex {
                pos: [0., 0.],
                uv: [0., 66./256.],
            },
            GuiTextureVertex {
                pos: [1., 0.],
                uv: [200./256., 66./256.],
            },
            GuiTextureVertex {
                pos: [1., 1.],
                uv: [200./256., 46./256.],
            },
            GuiTextureVertex {
                pos: [0., 1.],
                uv: [0., 46./256.],
            },
        ], &[0u16, 1, 2, 2, 3, 0]);

        GuiResources {
            font_pipeline: pipeline,
            font_texture: texture_bind_group,
            font_uniform_layout,
            font_map,
            glyph_coords,

            button_pipeline: gui_texture_pipeline,
            button_uniform_layout,
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
        let uniform_storage = UniformStorage::new(gpu, "Gui", &[
            (&self.font_uniform_layout, gui_spec.buttons.len(), "Text Bindings"),
            (&self.button_uniform_layout, gui_spec.buttons.len(), "Button Bindings"),
        ]);

        let buttons: Vec<_> = gui_spec.buttons
            .iter()
            .enumerate()
            .map(|(i, text)| Button {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
                baked_text: self.build_text(&gpu, text, true).unwrap(),
                text_uniform_offset: self.font_uniform_layout.offset_of(i),
                button_uniform_offset: self.button_uniform_layout.offset_of(i),
            })
            .collect();

        Gui {
            buttons,
            hovered_button_index: None,
            uniform_storage,
        }
    }
}

pub struct Button {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    baked_text: Mesh,
    text_uniform_offset: wgpu::DynamicOffset,
    button_uniform_offset: wgpu::DynamicOffset,
}

impl Button {
    const BUTTON_TEXTURE_ASPECT: f32 = 10.;
    const TEXT_SCALE: f32 = 60.0;

    fn build_background_uniform(&self, projection: Mat4, hovered: bool) -> ButtonBackgroundUniform {
        let model = Mat4::from_translation(Vec3::new(self.x, self.y, 0.))
            * Mat4::from_scale(Vec3::new(self.width, self.height, 1.));

        ButtonBackgroundUniform {
            mvp: (projection * model).to_cols_array(),
            y_offset: if hovered { 40./256. } else { 20./256. },
        }
    }

    fn build_text_mvp(&self, projection: &Mat4) -> Mat4 {
        let model = Mat4::from_translation(Vec3::new(self.x, self.y, 0.))
            * Mat4::from_scale(Vec3::new(Self::TEXT_SCALE, Self::TEXT_SCALE, 1.));
        *projection * model
    }

    fn recalculate_scale(&mut self, display_height: f32) {
        self.height = display_height / 10.;
        self.width = self.height * Self::BUTTON_TEXTURE_ASPECT;
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}

pub struct Gui {
    buttons: Vec<Button>,
    hovered_button_index: Option<usize>,
    uniform_storage: UniformStorage,
}

impl Gui {
    pub fn button(&mut self, id: usize) -> &mut Button {
        &mut self.buttons[id]
    }

    pub fn update_button_scales(&mut self, window_height: f32) {
        for button in &mut self.buttons {
            button.recalculate_scale(window_height);
        }
    }

    pub fn resize(&mut self, gpu: &GpuWrapper) {
        let width = gpu.window_size().width as f32;
        let height = gpu.window_size().height as f32;
        let projection = Mat4::orthographic_lh(
            0.,
            width,
            0.,
            height,
            -1.,
            1.,
        );

        for (i, button) in self.buttons.iter().enumerate() {
            let background_uniform = button.build_background_uniform(projection, Some(i) == self.hovered_button_index);
            self.uniform_storage.set_element(0, i, background_uniform);

            let text_uniform = button.build_text_mvp(&projection).to_cols_array();
            self.uniform_storage.set_element(1, i, text_uniform);
        }
        self.uniform_storage.update(gpu);
    }

    pub fn mouse_moved(&mut self, gpu: &GpuWrapper, position: PhysicalPosition<f32>) {
        let mouse_x = position.x as f32;
        let mouse_y = position.y as f32;

        let mut hovered_index = None;
        for (i, button) in self.buttons.iter().enumerate() {
            let inside = mouse_x > button.x && mouse_x < button.x + button.width
                && mouse_y > button.y && mouse_y < button.y + button.height;

            if inside {
                hovered_index = Some(i);
                break
            }
        }

        if hovered_index != self.hovered_button_index {
            self.hovered_button_index = hovered_index;
            self.resize(gpu);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, resources: &'a GuiResources) {
        render_pass.set_pipeline(&resources.button_pipeline);
        render_pass.set_bind_group(0, &resources.gui_texture, &[]);
        resources.gui_button_large.bind(render_pass);
        for button in &self.buttons {
            render_pass.set_bind_group(1, self.uniform_storage.bind_group(0), &[button.text_uniform_offset]);
            resources.gui_button_large.draw(render_pass);
        }

        render_pass.set_pipeline(&resources.font_pipeline);
        render_pass.set_bind_group(0, &resources.font_texture, &[]);
        for button in &self.buttons {
            render_pass.set_bind_group(1, self.uniform_storage.bind_group(1), &[button.button_uniform_offset]);
            button.baked_text.bind(render_pass);
            button.baked_text.draw(render_pass);
        }
    }
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
struct ButtonBackgroundUniform {
    mvp: [f32; 16],
    y_offset: f32,
}
