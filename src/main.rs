pub struct Scene;

impl miniquad::EventHandler for Scene {
    fn update(&mut self, _ctx: &mut miniquad::Context) {
    }

    fn draw(&mut self, ctx: &mut miniquad::Context) {
        ctx.begin_default_pass(Default::default());
        ctx.end_render_pass();
        ctx.commit_frame();
    }
}

fn main() {
    miniquad::start(miniquad::conf::Conf {
        window_title: "golden".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }, |_| Box::new(Scene));
}
