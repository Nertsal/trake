use crate::{
    model::*,
    prelude::*,
    render::game::{GameRender, GameRenderOptions},
};

pub struct GameState {
    context: Context,

    render: GameRender,
    render_options: GameRenderOptions,
    model: Model,
}

impl GameState {
    pub fn new(context: Context) -> Self {
        Self {
            render: GameRender::new(context.clone()),
            render_options: GameRenderOptions::default(),
            model: Model::new(context.assets.config.clone()),

            context,
        }
    }

    fn handle_key(&mut self, key: geng::Key) {
        match key {
            geng::Key::F2 => {
                self.render_options.show_colliders = !self.render_options.show_colliders;
            }
            _ => {}
        }
    }
}

impl geng::State for GameState {
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => self.handle_key(key),
            _ => {}
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        self.render
            .draw_game(&self.model, &self.render_options, framebuffer);
    }
}
