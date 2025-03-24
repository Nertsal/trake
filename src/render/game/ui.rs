use super::*;

impl GameRender {
    pub fn draw_game_ui(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let camera = &Camera2d {
            center: vec2::ZERO,
            rotation: Angle::ZERO,
            fov: 9.0,
        };

        // Score
        self.util.draw_text(
            format!("Score: {}", model.round_score),
            vec2(-7.0, 2.5),
            TextRenderOptions::new(1.0).align(vec2(0.0, 0.5)),
            camera,
            framebuffer,
        );
    }
}
