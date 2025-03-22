use crate::{model::*, prelude::*, render::game::GameRender};

pub struct GameState {
    context: Context,

    render: GameRender,
    model: Model,
}

impl GameState {
    pub fn new(context: Context) -> Self {
        Self {
            render: GameRender::new(context.clone()),
            model: Model::new(context.assets.config.clone()),

            context,
        }
    }
}

impl geng::State for GameState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        self.render.draw_game(&self.model, framebuffer);
    }
}
