mod gl;
mod graphics;

use glam::{Vec2, Mat4, Vec3};
use glfw::Context;
use image::GenericImageView;

#[repr(C)]
#[derive(Clone)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

pub struct Scene {
    text: Option<graphics::Mesh<Vertex>>,

    font_texture: graphics::PixelTexture,
    font_program: graphics::Program,
    font_mat4_uniform: graphics::MatrixUniform,
    font_map: Vec<char>,
    // Left, top, width
    glyph_coords: Vec<(f32, f32, f32)>,
}

impl Scene {
    pub fn new() -> Scene {
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

        let texture = graphics::PixelTexture::new();
        texture.set_data(&image);

        let program = graphics::Program::new(
            include_str!("../res/text.vsh"),
            include_str!("../res/text.fsh"),
        );

        let uniform = program.get_uniform("u_mvp");
        
        let mut scene = Scene {
            text: None,

            font_texture: texture,
            font_program: program,
            font_mat4_uniform: uniform,
            
            font_map: include_str!("../res/font.txt").to_owned().replace("\n", "").chars().collect(),
            glyph_coords: glyph_sizes,
        };
        
        if let Some(index) = scene.font_map.iter().position(|c| *c == ' ') {
            scene.glyph_coords[index + 32].2 = 4. / 256.;
        }
        scene
    }

    fn build_text(&self, text: &str) -> Option<graphics::Mesh<Vertex>> {
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

        let mut mesh = graphics::Mesh::new();
        mesh.set_data(&vertices, &indices);
        mesh.init_layout();
        mesh.add_layout::<Vec2>(0, 0);
        mesh.add_layout::<Vec2>(1, 4*2);
        Some(mesh)
    }

    fn update(&mut self) {
        if self.text.is_none() {
            self.text = self.build_text("This is some test Text!");
        }
    }

    fn draw(&self) {
        let (width, height) = (1280., 720.); // TODO
        let projection = Mat4::orthographic_rh(0., width, height, 0., -1., 1.);
        let view = Mat4::IDENTITY;
        let model = Mat4::from_scale(Vec3::new(700., 700., 1.));

        self.font_program.bind();

        if let Some(text) = &self.text {
            let mvp = projection * view * model;

            self.font_texture.bind();
            self.font_program.bind();
            self.font_mat4_uniform.set(&mvp);
            text.bind_and_render();
        } else {
            println!("nothing to render");
        }
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(1280, 730, "golden", glfw::WindowMode::Windowed)
        .unwrap();

    window.make_current();
    window.set_key_polling(true);
    graphics::init(&mut window);

    let mut scene = Scene::new();

    while !window.should_close() {
        window.swap_buffers();

        scene.update();
        scene.draw();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true);
                }

                _ => {},
            }
        }
    }
}
