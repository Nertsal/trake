mod actions;
mod ui;

use self::{actions::*, ui::GameUi};

use crate::{
    model::*,
    prelude::*,
    render::game::{GameRender, GameRenderOptions},
    ui::UiContext,
};

pub struct GameState {
    context: Context,
    ui_context: UiContext,

    render: GameRender,
    model: Model,
    ui: GameUi,
    ui_focused: bool,

    framebuffer_size: vec2<usize>,
    render_options: GameRenderOptions,
    game_texture: ugli::Texture,

    cursor_pos: vec2<f64>,
    cursor_world_pos: vec2<Coord>,
    cursor_grid_pos: vec2<ICoord>,

    place_rail_kind: RailKind,
    place_rotation: usize,
}

impl GameState {
    pub fn new(context: Context) -> Self {
        let mut game_texture =
            geng_utils::texture::new_texture(context.geng.ugli(), crate::GAME_RESOLUTION);
        game_texture.set_filter(ugli::Filter::Nearest);
        Self {
            render: GameRender::new(context.clone()),
            model: Model::new(context.clone(), context.assets.config.clone()),
            ui: GameUi::new(),
            ui_focused: false,

            framebuffer_size: vec2(1, 1),
            render_options: GameRenderOptions::default(),
            game_texture,

            cursor_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            cursor_grid_pos: vec2::ZERO,

            place_rail_kind: RailKind::Straight,
            place_rotation: 0,

            ui_context: UiContext::new(context.clone()),
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
            geng::Key::G => {
                self.model.grid_items.insert(GridItem {
                    position: self.cursor_grid_pos,
                    rail: None,
                    resource: Some(Resource::Coal),
                    wall: None,
                });
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
    fn update(&mut self, delta_time: f64) {
        let delta_time = r32(delta_time as f32);
        self.ui_context.update(delta_time.as_f32());

        self.model.update(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => self.handle_key(key),
            geng::Event::MousePress { button } => self.handle_mouse(button),
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
                self.cursor_pos = position;

                let game = self.ui.game.position;
                let position = position.as_f32() - game.bottom_left();
                self.cursor_world_pos = self
                    .model
                    .camera
                    .screen_to_world(game.size().as_f32(), position)
                    .as_r32();
                self.cursor_grid_pos = self.model.grid.world_to_grid(self.cursor_world_pos);
            }
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            _ => {}
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.ui_context.state.frame_start();
        self.ui_context.geometry.update(framebuffer.size());
        let actions = self.ui.layout(
            &self.model,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &mut self.ui_context,
        );
        self.ui_focused = !self.ui_context.can_focus();
        self.ui_context.frame_end();
        for action in actions {
            self.execute(action);
        }

        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(Color::try_from("#250f54").unwrap()),
            None,
            None,
        );

        let mut game_buffer =
            geng_utils::texture::attach_texture(&mut self.game_texture, self.context.geng.ugli());
        ugli::clear(
            &mut game_buffer,
            Some(Color::try_from("#250f54").unwrap()),
            None,
            None,
        );
        self.render
            .draw_game(&self.model, &self.render_options, &mut game_buffer);
        geng_utils::texture::DrawTexture::new(&self.game_texture)
            .fit(self.ui.game.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);

        self.render
            .draw_game_ui(&self.model, &self.ui_context, framebuffer);
    }
}
