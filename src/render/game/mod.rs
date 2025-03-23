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
        // Wall
        let bounds = Aabb2::from_corners(
            model.grid.gridf_to_world(vec2(0.5, 0.5).as_r32()),
            model
                .grid
                .gridf_to_world(model.config.map_size.map(|x| r32(x as f32 + 0.5))),
        )
        .extend_uniform(r32(0.15));
        self.util.draw_outline(
            &Collider::aabb(bounds),
            0.15,
            Color::try_from("#ab1f65").unwrap(),
            &model.camera,
            framebuffer,
        );
        // for (&pos, wall) in query!(model.grid_items, (&position, &wall.Get.Some)) {
        //     let position = model.grid.grid_to_world(pos);
        //     let texture = &self.context.assets.sprites.wall;
        //     self.context.geng.draw2d().draw2d(
        //         framebuffer,
        //         &model.camera,
        //         &draw2d::TexturedQuad::unit(&***texture)
        //             .scale(model.grid.cell_size.as_f32() / 2.0)
        //             .translate(position.as_f32()),
        //     );

        //     if options.show_colliders {
        //         self.util.draw_outline(
        //             &wall.collider,
        //             OUTLINE_WIDTH,
        //             Color::RED,
        //             &model.camera,
        //             framebuffer,
        //         );
        //     }
        // }

        // Rails
        for (&pos, rail) in query!(model.grid_items, (&position, &rail.Get.Some)) {
            let position = model.grid.grid_to_world(pos);
            let texture = match rail.orientation.kind {
                RailKind::Straight => &self.context.assets.sprites.rail_straight,
                RailKind::Left => &self.context.assets.sprites.rail_left,
            };
            self.context.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::TexturedQuad::unit(&***texture)
                    .scale(model.grid.cell_size.as_f32() / 2.0)
                    .rotate(Angle::from_degrees(90.0) * (rail.orientation.rotation as f32 - 1.0))
                    .translate(position.as_f32()),
            );
        }

        // Resources
        for (&pos, resource) in query!(model.grid_items, (&position, &resource.Get.Some)) {
            let position = model.grid.grid_to_world(pos);
            let texture = match resource {
                Resource::Coal => &self.context.assets.sprites.coal,
            };
            self.context.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::TexturedQuad::unit(&***texture)
                    .scale(model.grid.cell_size.as_f32() / 2.0)
                    .translate(position.as_f32()),
            );
        }

        // Train
        for block in &model.train.blocks {
            let size = match block.collider.shape {
                Shape::Circle { .. } => todo!(),
                Shape::Rectangle { width, height } => {
                    Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0)
                }
            };
            self.context.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::Quad::new(size, Color::try_from("#ffda45").unwrap())
                    .rotate(block.collider.rotation.map(R32::as_f32))
                    .translate(block.collider.position.as_f32()),
            );
            self.context.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::Quad::new(size, Color::try_from("#ff8142").unwrap())
                    .rotate(block.collider.rotation.map(R32::as_f32))
                    .translate(block.collider.position.as_f32() + vec2(-0.1, 0.1)),
            );

            // let draw =
            //     geng_utils::texture::DrawTexture::new(&self.context.assets.sprites.locomotive)
            //         .fit(target, vec2::splat(0.5));
            // self.context.geng.draw2d().draw2d(
            //     framebuffer,
            //     &model.camera,
            //     &draw2d::TexturedQuad::unit(draw.texture)
            //         .scale(draw.target.size())
            //         .rotate(block.collider.rotation.map(R32::as_f32) - Angle::from_degrees(90.0))
            //         .translate(draw.target.center()),
            // );

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
