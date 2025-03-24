use super::*;

use crate::ui::{geometry::Geometry, UiContext};

impl GameRender {
    pub fn draw_game_ui(
        &mut self,
        model: &Model,
        context: &UiContext,
        final_buffer: &mut ugli::Framebuffer,
    ) {
        self.mask_stack.update_size(final_buffer.size());
        if self.ui_texture.size() != final_buffer.size() {
            self.ui_texture =
                ugli::Texture::new_with(self.context.geng.ugli(), final_buffer.size(), |_| {
                    Rgba::BLACK
                });
            self.ui_depth = ugli::Renderbuffer::new(self.context.geng.ugli(), final_buffer.size());
            self.ui_texture.set_filter(ugli::Filter::Nearest);
        }

        let framebuffer = &mut ugli::Framebuffer::new(
            self.context.geng.ugli(),
            ugli::ColorAttachment::Texture(&mut self.ui_texture),
            ugli::DepthAttachment::Renderbuffer(&mut self.ui_depth),
        );

        let camera = &geng::PixelPerfectCamera;
        ugli::clear(framebuffer, Some(Color::TRANSPARENT_BLACK), Some(1.0), None);

        let geometry = Geometry::new();
        let geometry = RefCell::new(geometry);
        context.state.iter_widgets(
            |w| {
                geometry.borrow_mut().merge(w.draw_top(context));
            },
            |w| {
                geometry.borrow_mut().merge(w.draw(context));
            },
        );
        let geometry = geometry.into_inner();

        self.util
            .draw_geometry(&mut self.mask_stack, geometry, camera, framebuffer);

        self.context.geng.draw2d().textured_quad(
            final_buffer,
            camera,
            Aabb2::ZERO.extend_positive(final_buffer.size().as_f32()),
            &self.ui_texture,
            Color::WHITE,
        );
    }
}
