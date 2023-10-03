use crate::render::Renderer;

use super::ui::{Gui, Button};

pub struct TitleGui {
    buttons: Vec<Button>,
}

impl TitleGui {
    pub fn new(renderer: &Renderer) -> TitleGui {
        TitleGui {
            buttons: vec![
                Button::new(0.5, 0.5, "test", renderer),
            ],
        }
    }
}

impl Gui for TitleGui {
    fn buttons(&self) -> &[Button] {
        &self.buttons       
    }
}
