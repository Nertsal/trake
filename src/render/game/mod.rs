mod ui;

use super::{mask::MaskedStack, util::*};

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
    mask_stack: MaskedStack,
    ui_texture: ugli::Texture,
    ui_depth: ugli::Renderbuffer<ugli::DepthComponent>,
}

impl GameRender {
    pub fn new(context: Context) -> Self {
        let mut ui_texture = geng_utils::texture::new_texture(context.geng.ugli(), vec2(1, 1));
        let ui_depth = ugli::Renderbuffer::new(context.geng.ugli(), vec2(1, 1));
        ui_texture.set_filter(ugli::Filter::Nearest);
        Self {
            util: UtilRender::new(context.clone()),
            mask_stack: MaskedStack::new(&context.geng, &context.assets),
            ui_texture,
            ui_depth,
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
        let bounds = model.map_bounds.extend_uniform(r32(0.15));
        self.util.draw_outline(
            &Collider::aabb(bounds),
            0.15,
            Color::try_from("#ab1f65").unwrap(),
            &model.camera,
            framebuffer,
        );

        // Depo
        {
            let depo = &model.depo;
            let texture = &self.context.assets.sprites.depo;
            self.util.draw_texture_pp(
                texture,
                depo.position.as_f32(),
                vec2(0.5, 0.5),
                depo.rotation.map(R32::as_f32),
                1.0,
                &model.camera,
                framebuffer,
            );
        }

        // Resources
        for (&position, resource) in query!(model.grid_items, (&position, &resource.Get.Some)) {
            let texture = match resource {
                Resource::Coal => &self.context.assets.sprites.coal,
                Resource::Coin => &self.context.assets.sprites.coin,
                Resource::Diamond => &self.context.assets.sprites.diamond,
                Resource::PlusCent => &self.context.assets.sprites.plus_cent,
                Resource::GhostFuel => &self.context.assets.sprites.ghost_fuel,
            };
            self.util.draw_texture_pp(
                texture,
                position.as_f32(),
                vec2(0.5, 0.5),
                Angle::ZERO,
                1.0,
                &model.camera,
                framebuffer,
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

        // Particles
        #[derive(ugli::Vertex)]
        struct ParticleInstance {
            pub i_color: Rgba<f32>,
            pub i_model_matrix: mat3<f32>,
        }
        let instances: Vec<_> = query!(model.particles, (&kind, &position, &radius, &lifetime))
            .map(|(kind, position, radius, lifetime)| {
                let color = match kind {
                    ParticleKind::Steam => Color::try_from("#3d3957aa").unwrap(),
                    ParticleKind::Wall => Color::try_from("#ab1f65").unwrap(),
                    ParticleKind::WagonDestroyed => Color::try_from("#ffda45").unwrap(),
                    ParticleKind::Collect(resource) => match resource {
                        Resource::Coal => Color::try_from("#ff8142").unwrap(),
                        Resource::Coin => Color::try_from("#ffda45").unwrap(),
                        Resource::Diamond => Color::try_from("#49e7ec").unwrap(),
                        Resource::PlusCent => Color::try_from("#ffda45").unwrap(),
                        Resource::GhostFuel => Color::try_from("#ff8142").unwrap(),
                    },
                };
                let t = lifetime.get_ratio().as_f32().sqrt();
                let color = crate::util::with_alpha(color, t);
                let transform =
                    mat3::translate(position.as_f32()) * mat3::scale_uniform(radius.as_f32() * t);
                ParticleInstance {
                    i_color: color,
                    i_model_matrix: transform,
                }
            })
            .collect();
        let instances = ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), instances);
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.particles,
            ugli::DrawMode::TriangleFan,
            ugli::instanced(&self.util.unit_quad, &instances),
            (
                ugli::uniforms! {},
                model.camera.uniforms(framebuffer.size().as_f32()),
            ),
            ugli::DrawParameters { ..default() },
        );

        // Text
        for (text, position, size, color, lifetime) in query!(
            model.floating_texts,
            (&text, &position, &size, &color, &lifetime)
        ) {
            let t = lifetime.get_ratio().as_f32().sqrt();
            self.util.draw_text(
                text,
                position.as_f32(),
                TextRenderOptions::new(size.as_f32() * t).color(*color),
                &model.camera,
                framebuffer,
            );
        }
    }
}
