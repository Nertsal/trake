use super::*;

use crate::{prelude::Color, util::SubTexture};

#[derive(Clone)]
pub struct IconWidget {
    pub state: WidgetState,
    pub texture: SubTexture,
    pub color: Color,
    pub background: Option<IconBackground>,
}

#[derive(Debug, Clone)]
pub struct IconBackground {
    pub color: Color,
    pub kind: IconBackgroundKind,
}

#[derive(Debug, Clone, Copy)]
pub enum IconBackgroundKind {
    NineSlice,
    Circle,
}

impl IconWidget {
    pub fn new(texture: SubTexture) -> Self {
        Self {
            state: WidgetState::new(),
            texture: texture.clone(),
            color: Color::WHITE,
            background: None,
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for IconWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        let mut geometry = context.geometry.texture_pp(
            self.state.position.center(),
            self.color,
            1.0,
            &self.texture,
        );

        if let Some(bg) = &self.background {
            match bg.kind {
                IconBackgroundKind::NineSlice => {
                    let texture = //if width < 5.0 {
                        &context.context.assets.atlas.border_thin();
                    // } else {
                    //     &self.assets.sprites.fill
                    // };
                    geometry.merge(context.geometry.nine_slice(
                        self.state.position,
                        bg.color,
                        texture,
                    ));
                }
                IconBackgroundKind::Circle => {
                    geometry.merge(context.geometry.texture_pp(
                        self.state.position.center(),
                        bg.color,
                        1.0,
                        &context.context.assets.atlas.circle(),
                    ));
                }
            }
        }

        geometry
    }
}
