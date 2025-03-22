use crate::{
    model::*,
    prelude::*,
    render::game::{GameRender, GameRenderOptions},
};

pub struct GameState {
    context: Context,

    framebuffer_size: vec2<usize>,
    render: GameRender,
    render_options: GameRenderOptions,
    model: Model,

    cursor_pos: vec2<f64>,
    cursor_world_pos: vec2<Coord>,
    cursor_grid_pos: vec2<ICoord>,

    place_rail_kind: RailKind,
    place_rotation: usize,
}

impl GameState {
    pub fn new(context: Context) -> Self {
        Self {
            framebuffer_size: vec2(1, 1),
            render: GameRender::new(context.clone()),
            render_options: GameRenderOptions::default(),
            model: Model::new(context.assets.config.clone()),

            cursor_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            cursor_grid_pos: vec2::ZERO,

            place_rail_kind: RailKind::Straight,
            place_rotation: 0,

            context,
        }
    }

    fn handle_key(&mut self, key: geng::Key) {
        match key {
            geng::Key::F2 => {
                self.render_options.show_colliders = !self.render_options.show_colliders;
            }
            geng::Key::Q => {
                self.place_rotation = (self.place_rotation + 1) % 4;
            }
            geng::Key::E => {
                self.place_rotation = if self.place_rotation == 0 {
                    3
                } else {
                    self.place_rotation - 1
                };
            }
            geng::Key::Digit1 => {
                self.place_rail_kind = RailKind::Straight;
            }
            geng::Key::Digit2 => {
                self.place_rail_kind = RailKind::Left;
            }
            _ => {}
        }
    }

    fn handle_mouse(&mut self, _button: geng::MouseButton) {
        self.model.place_rail(
            self.cursor_grid_pos,
            RailOrientation {
                kind: self.place_rail_kind,
                rotation: self.place_rotation,
            },
        )
    }
}

impl geng::State for GameState {
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => self.handle_key(key),
            geng::Event::MousePress { button } => self.handle_mouse(button),
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
                self.cursor_world_pos = self
                    .model
                    .camera
                    .screen_to_world(self.framebuffer_size.as_f32(), position.as_f32())
                    .as_r32();
                self.cursor_grid_pos = self.model.grid.world_to_grid(self.cursor_world_pos);
            }
            _ => {}
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        self.render
            .draw_game(&self.model, &self.render_options, framebuffer);
    }
}
