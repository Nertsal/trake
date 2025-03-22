use super::util::*;

use crate::{model::*, prelude::*};

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

    pub fn draw_game(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        for block in &model.train.blocks {
            self.util
                .draw_collider(&block.collider, Color::GREEN, &model.camera, framebuffer);
        }
    }
}
