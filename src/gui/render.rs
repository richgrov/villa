use glam::{Mat4, Vec3};
use image::GenericImageView;
use winit::{dpi::PhysicalPosition, event::ElementState};

use crate::{gpu::{GpuWrapper, Mesh}, uniforms::{UniformStorage, UniformSpec}};

use super::BakedText;

pub struct GuiSpec {
    buttons: Vec<String>,
    images: Vec<(wgpu::BindGroup, Mesh)>,
}

impl GuiSpec {
    pub fn new() -> GuiSpec {
        GuiSpec {
            buttons: Vec::with_capacity(2),
            images: Vec::with_capacity(1),
        }
    }

    pub fn button(&mut self, text: &str) -> usize {
        let id = self.buttons.len();
        self.buttons.push(text.to_owned());
        id
    }

    pub fn image(&mut self, bind_group: wgpu::BindGroup, mesh: Mesh) -> usize {
        let id = self.images.len();
        self.images.push((bind_group, mesh));
        id
    }
}

pub struct GuiResources {
    font_pipeline: wgpu::RenderPipeline,
    font_texture: wgpu::BindGroup,
    font_uniform_layout: UniformSpec,
    pub(super) font_map: Vec<char>,
    pub(super) character_uv: Vec<CharacterUv>,

    pub sprite_pipeline: wgpu::RenderPipeline,
    gui_texture: wgpu::BindGroup,
    gui_button_large: Mesh,
    sprite_uniform_layout: UniformSpec,
}

#[derive(Clone, Debug)]
pub struct CharacterUv {
    left: f32,
    top: f32,
    width: f32,
}

impl CharacterUv {
    pub fn zero() -> CharacterUv {
        CharacterUv { left: 0., top: 0., width: 0. }
    }

    pub fn left(&self) -> f32 {
        self.left
    }

    pub fn top(&self) -> f32 {
        self.top
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}

impl GuiResources {
    pub fn new(gpu: &GpuWrapper) -> GuiResources {
        let font_uniform_layout = UniformSpec::new::<Mat4>(gpu, "Font Uniform Layout", wgpu::ShaderStages::VERTEX);
        let sprite_uniform_layout = UniformSpec::new::<SpriteUniform>(gpu, "Sprite Uniform Layout", wgpu::ShaderStages::VERTEX);

        let pipeline = gpu.create_pipeline::<FontVertex>(
            "Font",
            include_str!("../../res/font.wgsl"),
            &[gpu.generic_texture_layout(), font_uniform_layout.layout()],
            false,
        );

        let image = image::load_from_memory(include_bytes!("../../res/default.png")).unwrap();
        if image.width() % 16 != 0 {
            panic!("font texture should have 16 evenly-sized columns");
        }
        if image.height() % 16 != 0 {
            panic!("font texture should have 16 evenly-sized rows");
        }

        let cell_width = image.width() / 16;
        let cell_height = image.height() / 16;
        let mut character_uv = vec![CharacterUv::zero(); 256];
        let font_map: Vec<_> = include_str!("../../res/font.txt").to_owned().replace("\n", "").chars().collect();
        for cell_x in 0..16 {
            for cell_y in 0..16 {
                let uv = &mut character_uv[(cell_y * 16 + cell_x) as usize];
                uv.left = cell_x as f32 / 16.;
                uv.top = cell_y as f32 / 16.;

                'find_width: for x_pixel in (0..cell_width).rev() {
                    for y_pixel in 0..cell_height {
                        let image_x = x_pixel + cell_x * cell_width;
                        let image_y = y_pixel + cell_y * cell_height;
                        if image.get_pixel(image_x, image_y)[3] != 0 {
                            uv.width = (x_pixel + 1) as f32 / image.width() as f32;
                            break 'find_width
                        }
                    }
                }
            }
        }

        let texture_bind_group = gpu.create_texture(&image);

        if let Some(index) = font_map.iter().position(|c| *c == ' ') {
            character_uv[index + 32].width = 4. / 256.;
        }

        let gui_texture_pipeline = gpu.create_pipeline::<SpriteVertex>(
            "Sprite Pipeline",
            include_str!("../../res/sprite.wgsl"),
            &[gpu.generic_texture_layout(), sprite_uniform_layout.layout()],
            false,
        );
        let gui_image = image::load_from_memory(include_bytes!("../../res/gui.png")).unwrap();
        let gui_texture = gpu.create_texture(&gui_image);
        let gui_button_large = gpu.create_mesh(&[
            SpriteVertex {
                pos: [0., 0.],
                uv: [0., 66./256.],
            },
            SpriteVertex {
                pos: [1., 0.],
                uv: [200./256., 66./256.],
            },
            SpriteVertex {
                pos: [1., 1.],
                uv: [200./256., 46./256.],
            },
            SpriteVertex {
                pos: [0., 1.],
                uv: [0., 46./256.],
            },
        ], &[0u16, 1, 2, 2, 3, 0]);

        GuiResources {
            font_pipeline: pipeline,
            font_texture: texture_bind_group,
            font_uniform_layout,
            font_map,
            character_uv,

            sprite_pipeline: gui_texture_pipeline,
            sprite_uniform_layout,

            gui_texture,
            gui_button_large,
        }
    }

    pub fn build_gui(&self, gpu: &GpuWrapper, gui_spec: GuiSpec) -> Gui {
        let uniform_storage = UniformStorage::new(gpu, "Gui", &[
            (&self.sprite_uniform_layout, gui_spec.buttons.len() + gui_spec.images.len(), "Image Bindings"),
            (&self.font_uniform_layout, gui_spec.buttons.len(), "Text Bindings"),
        ]);

        let mut image_uniform_index = 0;
        let mut images = Vec::with_capacity(gui_spec.images.len());
        for (bind_group, mesh) in gui_spec.images {
            images.push(Image {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
                texture: bind_group,
                mesh,
                uniform_offset: self.sprite_uniform_layout.offset_of(image_uniform_index),
            });
            image_uniform_index += 1;
        }

        let mut buttons = Vec::with_capacity(gui_spec.buttons.len());
        for (i, text) in gui_spec.buttons.iter().enumerate() {
            buttons.push(Button {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
                baked_text: BakedText::new(&gpu, self, &text, true).unwrap(),
                text_uniform_offset: self.font_uniform_layout.offset_of(i),
                button_uniform_offset: self.sprite_uniform_layout.offset_of(image_uniform_index),
            });
            image_uniform_index += 1;
        }

        Gui {
            buttons,
            images,
            hovered_button_index: None,
            clicking_button_index: None,
            uniform_storage,
        }
    }
}

pub struct Image {
    x: f32,
    y: f32,
    pub width: f32,
    pub height: f32,
    texture: wgpu::BindGroup,
    mesh: Mesh,
    uniform_offset: wgpu::DynamicOffset,
}

impl Image {
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn build_uniform(&self, projection: Mat4) -> SpriteUniform {
        let model = Mat4::from_translation(Vec3::new(self.x, self.y, 0.))
            * Mat4::from_scale(Vec3::new(self.width, self.height, 1.));

        SpriteUniform {
            mvp: (projection * model).to_cols_array(),
            v_offset: 0.
        }
    }
}

pub struct Button {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    baked_text: BakedText,
    text_uniform_offset: wgpu::DynamicOffset,
    button_uniform_offset: wgpu::DynamicOffset,
}

impl Button {
    const BUTTON_TEXTURE_ASPECT: f32 = 10.;

    fn build_background_uniform(&self, projection: Mat4, hovered: bool) -> SpriteUniform {
        let model = Mat4::from_translation(Vec3::new(self.x, self.y, 0.))
            * Mat4::from_scale(Vec3::new(self.width, self.height, 1.));

        SpriteUniform {
            mvp: (projection * model).to_cols_array(),
            v_offset: if hovered { 40./256. } else { 20./256. },
        }
    }

    fn build_text_mvp(&self, projection: &Mat4) -> Mat4 {
        let scale = self.height / 2.;
        let centered_offset = self.width / 2. - (self.baked_text.width() * scale) / 2.;

        let model = Mat4::from_translation(Vec3::new(self.x + centered_offset, self.y + self.height * 0.25, 0.))
            * Mat4::from_scale(Vec3::new(scale, scale, 1.));
        *projection * model
    }

    fn recalculate_scale(&mut self, display_height: f32) {
        self.height = display_height * 0.08;
        self.width = self.height * Self::BUTTON_TEXTURE_ASPECT;
    }

    fn contains_point(&self, x: f32, y: f32) -> bool {
        x > self.x && x < self.x + self.width
            && y > self.y && y < self.y + self.height
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
    images: Vec<Image>,
    buttons: Vec<Button>,
    hovered_button_index: Option<usize>,
    clicking_button_index: Option<usize>,
    uniform_storage: UniformStorage,
}

impl Gui {
    pub fn image(&mut self, id: usize) -> &mut Image {
        &mut self.images[id]
    }

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

        for (i, image) in self.images.iter().enumerate() {
            let uniform = image.build_uniform(projection);
            self.uniform_storage.set_element(0, i, uniform);
        }

        for (i, button) in self.buttons.iter().enumerate() {
            let background_uniform = button.build_background_uniform(projection, Some(i) == self.hovered_button_index);
            self.uniform_storage.set_element(0, self.images.len() + i, background_uniform);

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
            if button.contains_point(mouse_x, mouse_y) {
                hovered_index = Some(i);
                break
            }
        }

        if hovered_index != self.hovered_button_index {
            self.hovered_button_index = hovered_index;
            self.resize(gpu);
        }
    }

    pub fn handle_click(&mut self, gpu: &GpuWrapper, state: ElementState, position: PhysicalPosition<f32>) -> Option<usize> {
        let mut clicked_index = None;
        for (i, button) in self.buttons.iter().enumerate() {
            if button.contains_point(position.x, position.y) {
                clicked_index = Some(i);
                break
            }
        }
        
        match state {
            ElementState::Pressed => {
                self.clicking_button_index = clicked_index;
            },
            ElementState::Released => {
                if self.clicking_button_index == clicked_index {
                    self.clicking_button_index = None;
                    return clicked_index
                }
            },
        }

        None
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, resources: &'a GuiResources) {
        render_pass.set_pipeline(&resources.sprite_pipeline);
        for image in &self.images {
            render_pass.set_bind_group(0, &image.texture, &[]);
            image.mesh.bind(render_pass);
            render_pass.set_bind_group(1, self.uniform_storage.bind_group(0), &[image.uniform_offset]);
            image.mesh.draw(render_pass);
        }

        render_pass.set_bind_group(0, &resources.gui_texture, &[]);
        resources.gui_button_large.bind(render_pass);
        for button in &self.buttons {
            render_pass.set_bind_group(1, self.uniform_storage.bind_group(0), &[button.button_uniform_offset]);
            resources.gui_button_large.draw(render_pass);
        }

        render_pass.set_pipeline(&resources.font_pipeline);
        render_pass.set_bind_group(0, &resources.font_texture, &[]);
        for button in &self.buttons {
            render_pass.set_bind_group(1, self.uniform_storage.bind_group(1), &[button.text_uniform_offset]);
            button.baked_text.bind(render_pass);
            button.baked_text.draw(render_pass);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct FontVertex {
    pub(super) pos: [f32; 2],
    pub(super) uv: [f32; 2],
    pub(super) color: [f32; 3],
}

const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x3];

impl crate::gpu::VertexAttribues for FontVertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &ATTRIBS
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

const SPRITE_VERTEX_ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

impl crate::gpu::VertexAttribues for SpriteVertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &SPRITE_VERTEX_ATTRIBS
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteUniform {
    pub mvp: [f32; 16],
    pub v_offset: f32,
}
