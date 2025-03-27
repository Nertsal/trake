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
        let palette = &self.context.assets.palette;

        // Wall
        let bounds = model.map_bounds.extend_uniform(r32(0.15));
        self.util.draw_outline(
            &Collider::aabb(bounds),
            0.15,
            palette.wall,
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

        // Resource collection indicator
        for wagon in &model.train.wagons {
            if let Some(collect) = &wagon.status.collect {
                if let Some(collecting) = &collect.collecting {
                    if let Some(&position) = get!(model.items, collecting.resource, (&position)) {
                        let radius = r32(0.2);
                        let width = r32(0.1);
                        let radius = radius + width * r32(2.0);

                        let t = collecting.completion.get_ratio();
                        let t = crate::util::smoothstep(t);

                        self.util.draw_circle_arc(
                            framebuffer,
                            &model.camera,
                            (mat3::translate(position) * mat3::scale_uniform(radius)).as_f32(),
                            crate::util::with_alpha(palette.range_circle, 0.5),
                            0.0,
                            Angle::from_degrees(90.0 - 360.0 * t.as_f32())
                                ..=Angle::from_degrees(90.0),
                        );
                    }
                }
            }
        }

        // Resources
        for (&position, resource) in query!(model.items, (&position, &resource.Get.Some)) {
            // let texture = match resource.kind {
            //     ResourceKind::Wood => &self.context.assets.sprites.wood,
            //     ResourceKind::Coal => &self.context.assets.sprites.coal,
            //     ResourceKind::Food => &self.context.assets.sprites.food,
            // };
            // self.util.draw_texture_pp(
            //     texture,
            //     position.as_f32(),
            //     vec2(0.5, 0.5),
            //     Angle::ZERO,
            //     1.0,
            //     &model.camera,
            //     framebuffer,
            // );

            let t = match resource.state {
                ResourceNodeState::Idle => r32(1.0),
                ResourceNodeState::Spawning(time) | ResourceNodeState::Despawning(time) => {
                    time.get_ratio()
                }
            };
            let t = crate::util::smoothstep(t);
            let radius = r32(0.2) * t;
            let color = palette
                .resources
                .get(&resource.kind)
                .copied()
                .unwrap_or(palette.default_color);

            self.util.draw_collider(
                &Collider::circle(position, radius),
                color,
                &model.camera,
                framebuffer,
            );
        }

        // Train
        for wagon in &model.train.wagons {
            let size = match wagon.collider.shape {
                Shape::Circle { .. } => todo!(),
                Shape::Rectangle { width, height } | Shape::RectangleOutline { width, height } => {
                    Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0)
                }
            };

            let offset = vec2(-0.1, 0.1);
            self.context.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::Quad::new(size, palette.wagon_bottom)
                    .rotate(wagon.collider.rotation.map(R32::as_f32))
                    .translate(wagon.collider.position.as_f32()),
            );

            self.context.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::Quad::new(size, palette.wagon_top)
                    .rotate(wagon.collider.rotation.map(R32::as_f32))
                    .translate(wagon.collider.position.as_f32() + offset),
            );

            if let Some(collect) = &wagon.status.collect {
                let color = palette
                    .resources
                    .get(&collect.stats.resource)
                    .copied()
                    .unwrap_or(palette.default_color);
                let collect_t = collect.total_stored() as f32 / collect.stats.capacity as f32;
                self.context.geng.draw2d().draw2d(
                    framebuffer,
                    &model.camera,
                    &draw2d::Quad::new(size, color)
                        .scale_uniform(collect_t * 0.9)
                        .rotate(wagon.collider.rotation.map(R32::as_f32))
                        .translate(-offset * (1.0 - collect_t))
                        .translate(wagon.collider.position.as_f32() + offset),
                );
            }

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
                    &wagon.collider,
                    OUTLINE_WIDTH,
                    Color::GREEN,
                    &model.camera,
                    framebuffer,
                );
            }
        }

        // Entities
        for (team, collider) in query!(model.entities, (&team, &collider)) {
            let color = match team {
                Some(Team::Player) => palette.team_player,
                Some(Team::Enemy) => palette.team_enemy,
                None => palette.team_neutral,
            };
            self.util
                .draw_collider(collider, color, &model.camera, framebuffer)
        }

        // Wagon range circles
        for wagon in &model.train.wagons {
            if let Some(collect) = &wagon.status.collect {
                // TODO: seemless circles (without gaps between pixels when drawing thin)
                self.util.draw_outline(
                    &Collider::circle(wagon.collider.position, collect.stats.range),
                    0.05,
                    palette.range_circle,
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
                    ParticleKind::Steam => palette.steam,
                    ParticleKind::Wall => palette.wall,
                    ParticleKind::WagonDestroyed => palette.wagon_bottom,
                    ParticleKind::WagonDamaged => palette.team_enemy,
                    ParticleKind::Collect(resource) => palette
                        .resources
                        .get(resource)
                        .copied()
                        .unwrap_or_else(|| Color::try_from("#ff00ff").unwrap()),
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
