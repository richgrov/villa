use glam::{Vec2, Mat4, Vec3};
use image::GenericImageView;
use miniquad::{BufferType, Buffer, Context, Bindings, Pipeline, VertexAttribute, VertexFormat, BufferLayout, UniformDesc, UniformType};

#[repr(C)]
#[derive(Clone)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

#[repr(C)]
struct Uniforms {
    u_mvp: Mat4,
}

pub struct Scene {
    pipeline: Pipeline,
    text: Option<Bindings>,

    font_texture: miniquad::Texture,
    font_map: Vec<char>,
    // Left, top, width
    glyph_coords: Vec<(f32, f32, f32)>,
}

impl Scene {
    pub fn new(ctx: &mut Context) -> Scene {
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

        let texture = miniquad::Texture::from_rgba8(
            ctx,
            image.width() as u16,
            image.height() as u16,
            &image.to_rgba8()
        );
        texture.set_filter(ctx, miniquad::FilterMode::Nearest);
        texture.set_wrap(ctx, miniquad::TextureWrap::Clamp);

        let shader = miniquad::Shader::new(
            ctx,
            include_str!("../res/text.vsh"),
            include_str!("../res/text.fsh"),
            miniquad::ShaderMeta {
                images: vec!["tex".to_string()],
                uniforms: miniquad::UniformBlockLayout {
                    uniforms: vec![UniformDesc::new("u_mvp", UniformType::Mat4)],
                },
            },
        ).unwrap();

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );
        
        let mut scene = Scene {
            pipeline,
            text: None,

            font_texture: texture,
            font_map: include_str!("../res/font.txt").to_owned().replace("\n", "").chars().collect(),
            glyph_coords: glyph_sizes,
        };
        
        if let Some(index) = scene.font_map.iter().position(|c| *c == ' ') {
            scene.glyph_coords[index + 32].2 = 4. / 256.;
        }
        scene
    }

    fn build_text(&self, ctx: &mut Context, text: &str) -> Option<Bindings> {
        let mut vertices = Vec::with_capacity(text.len() * 4);
        let mut indices = Vec::with_capacity(text.len() * 6);
        let mut x_offset = 0.;
        let mut index_offset = 0;

        for c in text.chars() {
            // Minecraft ignores the first 2 rows of characters so add 32 to the index
            let index = self.font_map.iter().position(|ch| *ch == c)? + 32;
            let (left, top, width) = self.glyph_coords[index];
            let height = 1./16.;

            vertices.extend_from_slice(&[
                Vertex { pos: Vec2::new(x_offset, 0.), uv: Vec2::new(left, top) },
                Vertex { pos: Vec2::new(x_offset + width, 0.), uv: Vec2::new(left + width, top) },
                Vertex { pos: Vec2::new(x_offset + width, height), uv: Vec2::new(left + width, top + height) },
                Vertex { pos: Vec2::new(x_offset, height), uv: Vec2::new(left, top + height) },
            ]);
            x_offset += width + 2. / 256.;

            indices.extend_from_slice(&[
                index_offset, index_offset + 1, index_offset + 2, index_offset + 2, index_offset + 3, index_offset
            ]);
            index_offset += 4;
        }

        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        Some(Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![self.font_texture],
        })
    }
}

impl miniquad::EventHandler for Scene {
    fn update(&mut self, ctx: &mut miniquad::Context) {
        if self.text.is_none() {
            self.text = self.build_text(ctx, "This is some test Text!");
        }
    }

    fn draw(&mut self, ctx: &mut miniquad::Context) {
        let (width, height) = ctx.screen_size();
        let projection = Mat4::orthographic_rh(0., width, height, 0., -1., 1.);
        let view = Mat4::IDENTITY;
        let model = Mat4::from_scale(Vec3::new(700., 700., 1.));

        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);

        if let Some(text) = &self.text {
            ctx.apply_bindings(text);

            let mvp = projection * view * model;
            ctx.apply_uniforms(&Uniforms {
                u_mvp: mvp,
            });
            ctx.draw(0, text.index_buffer.size() as i32, 1);
        } else {
            println!("nothing to render");
        }

        ctx.end_render_pass();
        ctx.commit_frame();
    }
}

fn main() {
    miniquad::start(miniquad::conf::Conf {
        window_title: "golden".to_owned(),
        ..Default::default()
    }, |ctx| Box::new(Scene::new(ctx)));
}

