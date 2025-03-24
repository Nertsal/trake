use super::*;

use crate::ui::{layout::AreaOps, widget::*};

pub struct GameUi {
    pub game: WidgetState,
}

impl GameUi {
    pub fn new() -> Self {
        Self {
            game: WidgetState::new(),
        }
    }

    pub fn layout(
        &mut self,
        model: &Model,
        screen: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Vec<GameAction> {
        let mut actions = Vec::new();

        let mut screen = screen.fit_aabb(vec2(16.0, 9.0), vec2(0.5, 0.5));
        let font_size = screen.height() * 0.04;
        let layout_size = screen.height() * 0.03;

        context.font_size = font_size;
        context.layout_size = layout_size;
        context.screen = screen;

        let left_bar = screen.cut_left(screen.width() / 3.0);
        let bottom_bar = screen.cut_bottom(font_size * 6.0);

        self.game.update(screen, context);

        actions
    }
}
