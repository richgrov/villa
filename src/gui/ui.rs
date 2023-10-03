use crate::render::{self, Renderer};

pub struct Button {
    x: f32,
    y: f32,
    text: String,
    baked_text: render::Mesh,
}

impl Button {
    pub fn new(x: f32, y: f32, text: &str, renderer: &Renderer) -> Button {
        Button {
            x,
            y,
            text: text.to_owned(),
            baked_text: renderer.font().build_text(renderer.gpu(), text, true).unwrap(),
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn mesh(&self) -> &render::Mesh {
        &self.baked_text
    }
}

pub trait Gui {
    fn buttons(&self) -> &[Button];
}
