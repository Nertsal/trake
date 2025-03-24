pub mod game;
pub mod mask;
pub mod util;

use crate::prelude::*;

pub struct SwapBuffer {
    ugli: Ugli,
    pub active: ugli::Texture,
    pub second: ugli::Texture,
}

impl SwapBuffer {
    pub fn new(ugli: &Ugli, size: vec2<usize>) -> Self {
        let new_texture = || {
            let mut texture = geng_utils::texture::new_texture(ugli, size);
            texture.set_filter(ugli::Filter::Nearest);
            texture
        };
        Self {
            ugli: ugli.clone(),
            active: new_texture(),
            second: new_texture(),
        }
    }

    pub fn size(&self) -> vec2<usize> {
        self.active.size()
    }

    pub fn update_size(&mut self, size: vec2<usize>) {
        if self.active.size() != size {
            let new_texture = || {
                let mut texture = geng_utils::texture::new_texture(&self.ugli, size);
                texture.set_filter(ugli::Filter::Nearest);
                texture
            };
            self.active = new_texture();
            self.second = new_texture();
        }
    }

    pub fn active_draw(&mut self) -> ugli::Framebuffer<'_> {
        geng_utils::texture::attach_texture(&mut self.active, &self.ugli)
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.active, &mut self.second);
    }
}
