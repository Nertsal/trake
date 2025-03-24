mod font;

pub use self::font::*;

use std::path::PathBuf;

use geng::prelude::*;
use geng_utils::gif::GifFrame;

#[derive(geng::asset::Load)]
pub struct LoadingAssets {
    #[load(path = "sprites/title.png", options(filter = "ugli::Filter::Nearest"))]
    pub title: ugli::Texture,
    #[load(path = "fonts/default.ttf")]
    pub font: Font,
    #[load(load_with = "load_gif(&manager, &base_path.join(\"sprites/loading_background.gif\"))")]
    pub background: Vec<GifFrame>,
}

fn load_gif(
    manager: &geng::asset::Manager,
    path: &std::path::Path,
) -> geng::asset::Future<Vec<GifFrame>> {
    let manager = manager.clone();
    let path = path.to_owned();
    async move {
        geng_utils::gif::load_gif(
            &manager,
            &path,
            geng_utils::gif::GifOptions {
                frame: geng::asset::TextureOptions {
                    filter: ugli::Filter::Nearest,
                    ..Default::default()
                },
            },
        )
        .await
    }
    .boxed_local()
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub atlas: Rc<SpritesAtlas>,
    pub config: crate::model::Config,
    pub controls: crate::game::Controls,
    pub sprites: SpriteAssets,
    pub shaders: ShaderAssets,
    pub fonts: FontAssets,
    pub sounds: SoundAssets,
}

trake_derive::texture_atlas!(pub SpritesAtlas {
    coin,
    ghost_fuel,
    plus_cent,
    speed,
    feather,

    white,
    circle,

    fill,
    fill_thin,
    fill_thinner,
    border,
    border_thin,
    border_thinner,
});

#[derive(geng::asset::Load)]
pub struct SpriteAssets {
    pub locomotive: Rc<PixelTexture>,
    pub rail_straight: Rc<PixelTexture>,
    pub rail_left: Rc<PixelTexture>,
    pub wall: Rc<PixelTexture>,
    pub depo: Rc<PixelTexture>,

    pub coal: Rc<PixelTexture>,
    pub coin: Rc<PixelTexture>,
    pub diamond: Rc<PixelTexture>,
    pub plus_cent: Rc<PixelTexture>,

    pub ghost_fuel: Rc<PixelTexture>,
    pub speed: Rc<PixelTexture>,
    pub feather_hat: Rc<PixelTexture>,
}

#[derive(geng::asset::Load)]
pub struct ShaderAssets {
    pub texture: Rc<ugli::Program>,
    pub solid: Rc<ugli::Program>,
    pub ellipse: Rc<ugli::Program>,
    pub masked: Rc<ugli::Program>,
    pub texture_ui: Rc<ugli::Program>,
}

#[derive(geng::asset::Load)]
pub struct FontAssets {
    pub default: Rc<Font>,
    pub pixel: Rc<Font>,
}

#[derive(geng::asset::Load)]
pub struct SoundAssets {
    pub choochoo: Rc<geng::Sound>,
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &()).await
    }
}

#[derive(Clone)]
pub struct PixelTexture {
    pub path: PathBuf,
    pub texture: Rc<ugli::Texture>,
}

impl Deref for PixelTexture {
    type Target = ugli::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl Debug for PixelTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PixelTexture")
            .field("path", &self.path)
            .field("texture", &"<texture data>")
            .finish()
    }
}

impl geng::asset::Load for PixelTexture {
    type Options = <ugli::Texture as geng::asset::Load>::Options;

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let path = path.to_owned();
        let texture = ugli::Texture::load(manager, &path, options);
        async move {
            let mut texture = texture.await?;
            texture.set_filter(ugli::Filter::Nearest);
            Ok(Self {
                path,
                texture: Rc::new(texture),
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}
