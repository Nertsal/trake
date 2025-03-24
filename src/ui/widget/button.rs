use super::*;

use crate::{context::ThemeColor, prelude::Name};

#[derive(Clone)]
pub struct ButtonWidget {
    pub text: TextWidget,
    pub bg_color: ThemeColor,
}

impl ButtonWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            bg_color: ThemeColor::Light,
        }
    }

    pub fn color(mut self, bg_color: ThemeColor) -> Self {
        self.bg_color = bg_color;
        self
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.text.update(position, context);
        self.text.options.color = context.theme().dark;
    }
}

impl Widget for ButtonWidget {
    simple_widget_state!(text);
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let state = &self.text.state;
        let width = self.text.options.size * 0.2;

        let mut geometry = self.text.draw(context);

        let position = state.position;
        let bg_color = theme.get_color(self.bg_color);
        geometry.merge(if state.pressed {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width), width, bg_color)
        } else if state.hovered {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width * 0.5), width, bg_color)
        } else {
            context.geometry.quad_fill(position, width, bg_color)
        });

        geometry
    }
}
