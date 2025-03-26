use super::{mask::MaskedStack, *};

use crate::{
    model::{Collider, Coord, Shape},
    ui::{geometry::Geometry, UiContext},
};

#[derive(Debug, Clone, Copy)]
pub struct TextRenderOptions {
    pub size: f32,
    pub align: vec2<f32>,
    pub color: Color,
    pub rotation: Angle,
}

impl TextRenderOptions {
    pub fn new(size: f32) -> Self {
        Self { size, ..default() }
    }

    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    pub fn align(self, align: vec2<f32>) -> Self {
        Self { align, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }

    pub fn update(&mut self, context: &UiContext) {
        self.size = context.font_size;
        self.color = context.theme().light;
    }
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            size: 1.0,
            align: vec2::splat(0.5),
            color: Color::WHITE,
            rotation: Angle::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DashRenderOptions {
    pub width: f32,
    pub dash_length: f32,
    pub space_length: f32,
}

pub struct UtilRender {
    context: Context,
    pub unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
}

impl UtilRender {
    pub fn new(context: Context) -> Self {
        Self {
            unit_quad: geng_utils::geometry::unit_quad_geometry(context.geng.ugli()),
            context,
        }
    }

    pub fn draw_geometry(
        &self,
        masked: &mut MaskedStack,
        geometry: Geometry,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        // log::debug!("Rendering geometry:");
        // log::debug!("^- triangles: {}", geometry.triangles.len() / 3);

        let framebuffer_size = framebuffer.size().as_f32();

        // Masked
        let mut frame = masked.pop_mask();
        for masked_geometry in geometry.masked {
            let mut masking = frame.start();
            self.draw_geometry(masked, masked_geometry.geometry, camera, &mut masking.color);
            masking.mask_quad(masked_geometry.clip_rect);
            frame.draw(
                masked_geometry.z_index,
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    depth_func: Some(ugli::DepthFunc::Less),
                    ..default()
                },
                framebuffer,
            );
        }
        masked.return_mask(frame);

        // Text
        for text in geometry.text {
            self.draw_text_with(
                text.text,
                text.position,
                text.z_index,
                text.options,
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    depth_func: Some(ugli::DepthFunc::LessOrEqual),
                    ..default()
                },
                camera,
                framebuffer,
            );
        }

        // Triangles & Textures
        let triangles =
            ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), geometry.triangles);
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.texture_ui,
            ugli::DrawMode::Triangles,
            &triangles,
            (
                ugli::uniforms! {
                    u_texture: self.context.assets.atlas.texture(),
                    u_model_matrix: mat3::identity(),
                    u_color: Color::WHITE,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_texture_pp(
        &self,
        texture: &ugli::Texture,
        position: vec2<f32>,
        align: vec2<f32>,
        rotation: Angle<f32>,
        pixel_scale: f32,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let draw = geng_utils::texture::DrawTexture::new(texture).pixel_perfect(
            position,
            align,
            pixel_scale,
            camera,
            framebuffer,
        );
        self.context.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::TexturedQuad::unit(draw.texture).transform(
                mat3::translate(draw.target.center())
                    * mat3::rotate(rotation)
                    * mat3::scale(draw.target.size() / 2.0),
            ),
        );
    }

    pub fn draw_nine_slice(
        &self,
        pos: Aabb2<f32>,
        color: Color,
        texture: &ugli::Texture,
        scale: f32,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let whole = Aabb2::ZERO.extend_positive(vec2::splat(1.0));

        // TODO: configurable
        let mid = Aabb2 {
            min: vec2(0.3, 0.3),
            max: vec2(0.7, 0.7),
        };

        let size = mid.min * texture.size().as_f32() * scale;
        let size = vec2(size.x.min(pos.width()), size.y.min(pos.height()));

        let tl = Aabb2::from_corners(mid.top_left(), whole.top_left());
        let tm = Aabb2::from_corners(mid.top_left(), vec2(mid.max.x, whole.max.y));
        let tr = Aabb2::from_corners(mid.top_right(), whole.top_right());
        let rm = Aabb2::from_corners(mid.top_right(), vec2(whole.max.x, mid.min.y));
        let br = Aabb2::from_corners(mid.bottom_right(), whole.bottom_right());
        let bm = Aabb2::from_corners(mid.bottom_right(), vec2(mid.min.x, whole.min.y));
        let bl = Aabb2::from_corners(mid.bottom_left(), whole.bottom_left());
        let lm = Aabb2::from_corners(mid.bottom_left(), vec2(whole.min.x, mid.max.y));

        let slices: Vec<draw2d::TexturedVertex> = [tl, tm, tr, rm, br, bm, bl, lm, mid]
            .into_iter()
            .flat_map(|slice| {
                let [a, b, c, d] = slice.corners().map(|a_vt| {
                    let a_pos = vec2(
                        if a_vt.x == mid.min.x {
                            pos.min.x + size.x
                        } else if a_vt.x == mid.max.x {
                            pos.max.x - size.x
                        } else {
                            pos.min.x + pos.width() * a_vt.x
                        },
                        if a_vt.y == mid.min.y {
                            pos.min.y + size.y
                        } else if a_vt.y == mid.max.y {
                            pos.max.y - size.y
                        } else {
                            pos.min.y + pos.height() * a_vt.y
                        },
                    );
                    draw2d::TexturedVertex {
                        a_pos,
                        a_color: Color::WHITE,
                        a_vt,
                    }
                });
                [a, b, c, a, c, d]
            })
            .collect();
        let slices = ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), slices);

        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.texture,
            ugli::DrawMode::Triangles,
            &slices,
            (
                ugli::uniforms! {
                    u_model_matrix: mat3::identity(),
                    u_color: color,
                    u_texture: texture,
                },
                camera.uniforms(framebuffer.size().as_f32()),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
        );

        // self.geng
        //     .draw2d()
        //     .textured_quad(framebuffer, camera, pos, texture, color);
    }

    pub fn draw_text(
        &self,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        options: TextRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_text_with(
            text,
            position,
            0.0,
            options,
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
            camera,
            framebuffer,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_text_with(
        &self,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        z_index: f32,
        mut options: TextRenderOptions,
        params: ugli::DrawParameters,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let text = text.as_ref();
        let font = &self.context.assets.fonts.pixel;
        let framebuffer_size = framebuffer.size().as_f32();

        let position = position.map(Float::as_f32);
        let position = crate::util::world_to_screen(camera, framebuffer_size, position);

        let scale = crate::util::world_to_screen(
            camera,
            framebuffer_size,
            vec2::splat(std::f32::consts::FRAC_1_SQRT_2),
        ) - crate::util::world_to_screen(camera, framebuffer_size, vec2::ZERO);
        options.size *= scale.len();
        let font_size = options.size * 0.6; // TODO: could rescale all dependent code but whatever

        let mut position = position;
        for line in text.lines() {
            let measure = font.measure(line, font_size);
            let size = measure.size();
            let align = size * (options.align - vec2::splat(0.5)); // Centered by default
            let descent = -font.descent() * font_size;
            let align = vec2(
                measure.center().x + align.x,
                descent + (measure.max.y - descent) * options.align.y,
            );

            let transform = mat3::translate(position)
                * mat3::rotate(options.rotation)
                * mat3::translate(-align);

            font.draw_with(
                framebuffer,
                line,
                z_index,
                font_size,
                options.color,
                transform,
                params.clone(),
            );
            position.y -= options.size; // NOTE: larger than text size to space out better
        }
    }

    pub fn draw_circle_cut(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        transform: mat3<f32>,
        color: Color,
        cut: f32,
    ) {
        self.draw_circle_arc(
            framebuffer,
            camera,
            transform,
            color,
            cut,
            Angle::from_radians(-std::f32::consts::PI)..=Angle::from_radians(std::f32::consts::PI),
        );
    }

    pub fn draw_circle_arc(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        transform: mat3<f32>,
        color: Color,
        cut: f32,
        range: RangeInclusive<Angle>,
    ) {
        let arc_min = range.start().as_radians();
        let arc_max = range.end().as_radians();
        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.ellipse,
            ugli::DrawMode::TriangleFan,
            &self.unit_quad,
            (
                ugli::uniforms! {
                    u_model_matrix: transform,
                    u_color: color,
                    u_framebuffer_size: framebuffer_size,
                    u_inner_cut: cut,
                    u_arc_min: arc_min,
                    u_arc_max: arc_max,
                },
                camera.uniforms(framebuffer_size.map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: None,
                ..Default::default()
            },
        );
    }

    fn draw_chain(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        chain: &draw2d::Chain,
    ) {
        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.solid,
            ugli::DrawMode::Triangles,
            &ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), chain.vertices.clone()),
            (
                ugli::uniforms! {
                    u_color: Rgba::WHITE,
                    u_framebuffer_size: framebuffer_size,
                    u_model_matrix: chain.transform,
                },
                camera.uniforms(framebuffer_size.map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: None,
                ..Default::default()
            },
        );
    }

    fn draw_segment(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        segment: &draw2d::Segment,
    ) {
        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.solid,
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), segment.vertices.clone()),
            (
                ugli::uniforms! {
                    u_color: Rgba::WHITE,
                    u_framebuffer_size: framebuffer_size,
                    u_model_matrix: segment.transform,
                },
                camera.uniforms(framebuffer_size.map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: None,
                ..Default::default()
            },
        );
    }

    pub fn draw_collider(
        &self,
        collider: &Collider,
        color: Rgba<f32>,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.draw_circle_cut(
                    framebuffer,
                    camera,
                    mat3::translate(collider.position.as_f32())
                        * mat3::scale_uniform(radius.as_f32()),
                    color,
                    0.0,
                );
            }
            Shape::Rectangle { width, height } => {
                self.context.geng.draw2d().draw2d_transformed(
                    framebuffer,
                    camera,
                    &draw2d::Quad::new(
                        Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0),
                        color,
                    ),
                    (mat3::translate(collider.position) * mat3::rotate(collider.rotation)).as_f32(),
                );
            }
            Shape::RectangleOutline { width, height } => {
                self.draw_outline(
                    &Collider {
                        position: collider.position,
                        rotation: collider.rotation,
                        shape: Shape::Rectangle { width, height },
                    },
                    0.1,
                    color,
                    camera,
                    framebuffer,
                );
            }
        }
    }

    pub fn draw_outline(
        &self,
        collider: &Collider,
        outline_width: f32,
        color: Rgba<f32>,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.draw_circle_cut(
                    framebuffer,
                    camera,
                    mat3::translate(collider.position.as_f32())
                        * mat3::scale_uniform(radius.as_f32()),
                    color,
                    (radius.as_f32() - outline_width) / radius.as_f32(),
                );
            }
            Shape::Rectangle { width, height } | Shape::RectangleOutline { width, height } => {
                let [a, b, c, d] = Aabb2::ZERO
                    .extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0)
                    .extend_uniform(-outline_width / 2.0)
                    .corners();
                let m = (a + b) / 2.0;
                self.draw_chain(
                    framebuffer,
                    camera,
                    &draw2d::Chain::new(
                        Chain::new(vec![m, b, c, d, a, m]),
                        outline_width,
                        color,
                        1,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
        }
    }

    pub fn draw_dashed_movement(
        &self,
        chain: &[draw2d::ColoredVertex],
        options: &DashRenderOptions,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let vertices: Vec<_> = chain
            .windows(2)
            .flat_map(|segment| {
                let (a, b) = (segment[0], segment[1]);
                let delta = b.a_pos - a.a_pos;
                let delta_len = delta.len();
                let direction = if delta_len.approx_eq(&0.0) {
                    return None;
                } else {
                    delta / delta_len
                };
                let b = draw2d::ColoredVertex {
                    a_pos: a.a_pos + direction * options.dash_length,
                    a_color: Color::lerp(a.a_color, b.a_color, options.dash_length / delta_len),
                };
                Some(build_segment(a, b, options.width))
            })
            .flatten()
            .collect();

        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.solid,
            ugli::DrawMode::Triangles,
            &ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), vertices),
            (
                ugli::uniforms! {
                    u_color: Rgba::WHITE,
                    u_framebuffer_size: framebuffer_size,
                    u_model_matrix: mat3::identity(),
                },
                camera.uniforms(framebuffer_size.map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: None,
                ..Default::default()
            },
        );
    }

    pub fn draw_dashed_chain(
        &self,
        chain: &[draw2d::ColoredVertex],
        options: &DashRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let mut dash_full_left = 0.0;
        let vertices: Vec<_> = chain
            .windows(2)
            .flat_map(|segment| {
                let segment = (segment[0], segment[1]);
                let (vertices, left) = self.build_dashed_segment(segment, options, dash_full_left);
                dash_full_left = left;
                vertices
            })
            .collect();

        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.context.assets.shaders.solid,
            ugli::DrawMode::Triangles,
            &ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), vertices),
            (
                ugli::uniforms! {
                    u_color: Rgba::WHITE,
                    u_framebuffer_size: framebuffer_size,
                    u_model_matrix: mat3::identity(),
                },
                camera.uniforms(framebuffer_size.map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: None,
                ..Default::default()
            },
        );
    }

    /// Draws a dashed segment.
    /// Returns the unrendered length of the last dash.
    fn build_dashed_segment(
        &self,
        mut segment: (draw2d::ColoredVertex, draw2d::ColoredVertex),
        options: &DashRenderOptions,
        dash_full_left: f32,
    ) -> (Vec<draw2d::ColoredVertex>, f32) {
        let mut vertices = vec![];

        let delta = segment.1.a_pos - segment.0.a_pos;
        let delta_len = delta.len();
        let direction_norm = if delta.len().approx_eq(&0.0) {
            return (vertices, dash_full_left);
        } else {
            delta / delta_len
        };

        if dash_full_left > 0.0 {
            // Finish drawing the previous dash and offset current segment
            let dash_full_length = dash_full_left.min(delta_len);
            let dash_length = dash_full_left - options.space_length;
            if dash_length > 0.0 {
                // Finish dash
                let dash_length = dash_length.min(dash_full_length);
                let dash_end = draw2d::ColoredVertex {
                    a_pos: segment.0.a_pos + direction_norm * dash_length,
                    a_color: Color::lerp(
                        segment.0.a_color,
                        segment.1.a_color,
                        dash_length / delta_len,
                    ),
                };
                vertices.extend(build_segment(segment.0, dash_end, options.width));
            }

            // Finish space
            let dash_left = dash_full_left - dash_full_length;
            if dash_left > 0.0 {
                return (vertices, dash_left);
            }

            // Offset
            segment.0.a_pos += dash_full_length * direction_norm;
            segment.0.a_color = Color::lerp(
                segment.0.a_color,
                segment.1.a_color,
                dash_full_length / delta_len,
            );
        }

        let full_length = options.dash_length + options.space_length;

        // Recalculate delta
        let delta_len = (segment.1.a_pos - segment.0.a_pos).len();
        let dashes = (delta_len / full_length).floor() as usize;
        for i in 0..dashes {
            let dash_start = draw2d::ColoredVertex {
                a_pos: segment.0.a_pos + direction_norm * i as f32 * full_length,
                a_color: Color::lerp(
                    segment.0.a_color,
                    segment.1.a_color,
                    full_length * i as f32 / delta_len,
                ),
            };
            let dash_end = draw2d::ColoredVertex {
                a_pos: dash_start.a_pos + direction_norm * options.dash_length,
                a_color: Color::lerp(
                    segment.0.a_color,
                    segment.1.a_color,
                    (full_length * i as f32 + options.dash_length) / delta_len,
                ),
            };
            vertices.extend(build_segment(dash_start, dash_end, options.width));
        }

        let last_start = draw2d::ColoredVertex {
            a_pos: segment.0.a_pos + direction_norm * dashes as f32 * full_length,
            a_color: Color::lerp(
                segment.0.a_color,
                segment.1.a_color,
                dashes as f32 * full_length / delta_len,
            ),
        };
        let last_len = (segment.1.a_pos - last_start.a_pos).len();
        let dash_len = last_len.min(options.dash_length);
        let last_end = draw2d::ColoredVertex {
            a_pos: last_start.a_pos + direction_norm * dash_len,
            a_color: Color::lerp(
                segment.0.a_color,
                segment.1.a_color,
                (dashes as f32 * full_length + dash_len) / delta_len,
            ),
        };
        vertices.extend(build_segment(last_start, last_end, options.width));

        (vertices, full_length - last_len)
    }
}

pub fn additive() -> ugli::BlendMode {
    ugli::BlendMode::combined(ugli::ChannelBlendMode {
        src_factor: ugli::BlendFactor::One,
        dst_factor: ugli::BlendFactor::One,
        equation: ugli::BlendEquation::Add,
    })
}

fn build_segment(
    start: draw2d::ColoredVertex,
    end: draw2d::ColoredVertex,
    width: f32,
) -> [draw2d::ColoredVertex; 6] {
    use draw2d::ColoredVertex;

    let half_width = width / 2.0;
    let normal = (end.a_pos - start.a_pos).normalize_or_zero().rotate_90();
    let a = ColoredVertex {
        a_pos: start.a_pos - normal * half_width,
        a_color: start.a_color,
    };
    let b = ColoredVertex {
        a_pos: start.a_pos + normal * half_width,
        a_color: start.a_color,
    };
    let c = ColoredVertex {
        a_pos: end.a_pos - normal * half_width,
        a_color: end.a_color,
    };
    let d = ColoredVertex {
        a_pos: end.a_pos + normal * half_width,
        a_color: end.a_color,
    };
    [a, b, c, b, d, c]
}
