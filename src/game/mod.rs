mod actions;
mod ui;

use self::{actions::*, ui::GameUi};

use crate::{
    model::*,
    prelude::*,
    render::{
        game::{GameRender, GameRenderOptions},
        SwapBuffer,
    },
    ui::UiContext,
};

use geng_utils::key::EventKey;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Controls {
    pub turn_left: Vec<EventKey>,
    pub turn_right: Vec<EventKey>,
    pub launch: Vec<EventKey>,
}

pub struct GameState {
    context: Context,
    ui_context: UiContext,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,

    render: GameRender,
    model: Model,
    ui: GameUi,
    ui_focused: bool,

    framebuffer_size: vec2<usize>,
    render_options: GameRenderOptions,
    pixel_buffer: SwapBuffer,
    post_buffer: SwapBuffer,

    cursor_pos: vec2<f64>,
    cursor_world_pos: vec2<Coord>,

    player_input: PlayerInput,
}

impl GameState {
    pub fn new(context: Context) -> Self {
        context.music.play(&context.assets.sounds.tootuh);
        Self {
            render: GameRender::new(context.clone()),
            model: Model::new(context.clone(), context.assets.config.clone()),
            ui: GameUi::new(),
            ui_focused: false,

            framebuffer_size: vec2(1, 1),
            render_options: GameRenderOptions::default(),
            pixel_buffer: SwapBuffer::new(context.geng.ugli(), crate::GAME_RESOLUTION),
            post_buffer: SwapBuffer::new(context.geng.ugli(), vec2(1, 1)),

            cursor_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,

            player_input: PlayerInput::default(),

            ui_context: UiContext::new(context.clone()),
            unit_quad: geng_utils::geometry::unit_quad_geometry(context.geng.ugli()),
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

    fn handle_mouse(&mut self, button: geng::MouseButton) {
        if let geng::MouseButton::Left = button {
            self.left_mouse();
        }
    }

    fn left_mouse(&mut self) {
        if let Phase::Finished = self.model.phase {
            if let Some(i) = self
                .model
                .tunnels
                .iter()
                .position(|tunnel| tunnel.collider.contains(self.cursor_world_pos))
            {
                self.model.choose_tunnel(i);
            }
        }
    }
}

impl geng::State for GameState {
    fn update(&mut self, delta_time: f64) {
        let delta_time = r32(delta_time as f32);
        self.ui_context.update(delta_time.as_f32());

        let mut input = std::mem::take(&mut self.player_input);
        let controls = &self.context.assets.controls;
        let window = self.context.geng.window();
        if geng_utils::key::is_key_pressed(window, &controls.turn_left) {
            input.turn += r32(1.0);
        } else if geng_utils::key::is_key_pressed(window, &controls.turn_right) {
            input.turn -= r32(1.0);
        }
        self.model.update(delta_time, input);
    }

    fn handle_event(&mut self, event: geng::Event) {
        let controls = &self.context.assets.controls;
        if geng_utils::key::is_event_press(&event, &controls.launch) {
            self.execute(GameAction::LaunchTrain);
        }

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
            }
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            _ => {}
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.post_buffer.update_size(framebuffer.size());

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

        let palette = &self.context.assets.palette;

        self.framebuffer_size = framebuffer.size();
        let bg_color = palette.background;
        ugli::clear(framebuffer, Some(bg_color), None, None);

        let pixel_buffer = &mut self.pixel_buffer.active_draw();
        ugli::clear(pixel_buffer, Some(bg_color), None, None);
        self.render
            .draw_game(&self.model, &self.render_options, pixel_buffer);

        let post_buffer = &mut self.post_buffer.active_draw();
        ugli::clear(post_buffer, Some(bg_color), None, None);

        {
            // Pixel perfect
            let pos = self.ui.game.position.center();
            let size = self.pixel_buffer.size() * 3;
            let align = vec2(0.5, 0.5);
            let align_size = (size.as_f32() * align).map(f32::fract);
            let pos = pos.map(f32::floor) + align_size;
            let screen_aabb = Aabb2::point(pos).extend_symmetric(size.as_f32() / 2.0);
            self.context.geng.draw2d().textured_quad(
                post_buffer,
                &geng::PixelPerfectCamera,
                screen_aabb,
                &self.pixel_buffer.active,
                Color::WHITE,
            );
        }

        self.render
            .draw_game_ui(&self.model, &self.ui_context, post_buffer);

        // Background
        // {
        //     self.post_buffer.swap();
        //     let post_buffer = &mut geng_utils::texture::attach_texture(
        //         &mut self.post_buffer.active,
        //         self.context.geng.ugli(),
        //     );
        //     let world_matrix = (self
        //         .model
        //         .camera
        //         .projection_matrix(post_buffer.size().as_f32())
        //         * self.model.camera.view_matrix())
        //     .inverse();
        //     ugli::draw(
        //         post_buffer,
        //         &self.context.assets.shaders.background,
        //         ugli::DrawMode::TriangleFan,
        //         &self.unit_quad,
        //         ugli::uniforms! {
        //             u_texture: &self.post_buffer.second,
        //             u_time: self.model.real_time.as_f32(),
        //             u_mask_color: bg_color,
        //             u_mask2_color: bg_color,
        //             u_world_matrix: world_matrix,
        //         },
        //         ugli::DrawParameters {
        //             blend_mode: Some(ugli::BlendMode::straight_alpha()),
        //             ..default()
        //         },
        //     );
        // }

        // Post
        self.context.geng.draw2d().textured_quad(
            framebuffer,
            &geng::PixelPerfectCamera,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &self.post_buffer.active,
            Color::WHITE,
        );
    }
}
