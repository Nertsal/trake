use super::*;

use crate::{
    prelude::{Color, Name},
    util::SubTexture,
};

#[derive(Clone)]
pub struct ButtonWidget {
    pub text: TextWidget,
    pub bg_color: Color,
}

impl ButtonWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            bg_color: Color::WHITE,
        }
    }

    pub fn color(mut self, bg_color: Color) -> Self {
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
        let state = &self.text.state;
        let width = self.text.options.size * 0.2;

        let mut geometry = self.text.draw(context);

        let position = state.position;
        let bg_color = self.bg_color;
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

#[derive(Clone)]
pub struct IconButtonWidget {
    pub state: WidgetState,
    pub icon: IconWidget,
    pub light_color: Color,
    pub dark_color: Color,
}

impl IconButtonWidget {
    pub fn new(
        texture: SubTexture,
        light_color: Color,
        dark_color: Color,
        bg_kind: IconBackgroundKind,
    ) -> Self {
        let mut icon = IconWidget::new(texture);
        icon.background = Some(IconBackground {
            color: Color::BLACK,
            kind: bg_kind,
        });
        Self {
            state: WidgetState::new(),
            icon,
            light_color,
            dark_color,
        }
    }

    pub fn new_normal(texture: SubTexture) -> Self {
        Self::new(
            texture,
            Color::try_from("#ab1f65").unwrap(),
            Color::try_from("#fff7f8").unwrap(),
            IconBackgroundKind::NineSlice,
        )
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        self.icon.update(position, context);

        let mut light = self.light_color;
        let mut dark = self.dark_color;
        if self.state.hovered {
            std::mem::swap(&mut dark, &mut light);
        }

        if let Some(bg) = &mut self.icon.background {
            bg.color = dark;
        }
    }
}

impl Widget for IconButtonWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        self.icon.draw(context)
    }
}
