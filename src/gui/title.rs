use crate::render::Renderer;

use super::{ui::Gui, GuiBuilder};

pub struct TitleGui {
    pub gui: Gui,
}

impl TitleGui {
    pub fn new(renderer: &Renderer) -> TitleGui {
        TitleGui {
            gui: GuiBuilder::new(renderer)
                .button(0.3, 0.8, "Singleplayer")
                .button(0.3, 0.6, "Multiplayer")
                .button(0.3, 0.4, "Options")
                .button(0.3, 0.2, "Quit")
                .build()
        }
    }
}
