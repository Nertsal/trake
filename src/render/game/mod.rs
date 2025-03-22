use super::util::*;

use crate::{model::*, prelude::*};

const OUTLINE_WIDTH: f32 = 0.1;

#[derive(Debug, Clone)]
pub struct GameRenderOptions {
    pub show_colliders: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for GameRenderOptions {
    fn default() -> Self {
        Self {
            show_colliders: false,
        }
    }
}

pub struct GameRender {
    context: Context,
    util: UtilRender,
}

impl GameRender {
    pub fn new(context: Context) -> Self {
        Self {
            util: UtilRender::new(context.clone()),
            context,
        }
    }

    pub fn draw_game(
        &mut self,
        model: &Model,
        options: &GameRenderOptions,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        for block in &model.train.blocks {
            let shape = match block.collider.shape {
                Shape::Circle { .. } => todo!(),
                Shape::Rectangle { width, height } => {
                    Shape::rectangle(vec2(width, height) * vec2(1.6, 1.3).as_r32())
                }
            };
            let collider = Collider {
                shape,
                ..block.collider
            };
            let draw =
                geng_utils::texture::DrawTexture::new(&self.context.assets.sprites.locomotive)
                    .fit(collider.compute_aabb().map(R32::as_f32), vec2::splat(0.5));
            self.context.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::TexturedQuad::unit(draw.texture)
                    .scale(draw.target.size())
                    .rotate(collider.rotation.map(R32::as_f32) - Angle::from_degrees(90.0))
                    .translate(draw.target.center()),
            );

            if options.show_colliders {
                self.util.draw_outline(
                    &block.collider,
                    OUTLINE_WIDTH,
                    Color::GREEN,
                    &model.camera,
                    framebuffer,
                );
            }
        }
    }
}
