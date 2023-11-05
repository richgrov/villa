use crate::gpu::GpuWrapper;

use super::{GuiRenderer, GuiSpec, Gui};

pub struct TitleGui {
    pub gui: Gui,
}

impl TitleGui {
    pub fn new(gpu: &GpuWrapper, gui_renderer: &GuiRenderer) -> TitleGui {
        let gui = GuiSpec::new()
            .button(0.3, 0.8, "Singleplayer")
            .button(0.3, 0.6, "Multiplayer")
            .button(0.3, 0.4, "Options")
            .button(0.3, 0.2, "Quit");

        TitleGui {
            gui: gui_renderer.build_gui(gpu, &gui),
        }
    }
}
