mod music;

pub use self::music::*;

use crate::prelude::*;

#[derive(Clone)]
pub struct Context {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub music: Rc<MusicManager>,
    options: Rc<RefCell<Options>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Options {
    pub theme: Theme,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            master_volume: 0.5,
            music_volume: 1.0,
            sfx_volume: 1.0,
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub enum ThemeColor {
    Dark,Light,Highlight,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub dark: Color,
    pub light: Color,
    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            dark: Color::try_from("#000000").unwrap(),
            light: Color::try_from("#ffffff").unwrap(),
            highlight: Color::try_from("#00ffff").unwrap(),
        }
    }
}

impl Theme {
    pub fn get_color(&self, color: ThemeColor) -> Color {
        match color {
            ThemeColor::Dark => self.dark,
            ThemeColor::Light => self.light,
            ThemeColor::Highlight => self.highlight,
        }
    }
}

impl Context {
    pub async fn new(geng: &Geng, assets: &Rc<Assets>) -> Result<Self> {
        let options: Options = preferences::load(crate::OPTIONS_STORAGE).unwrap_or_default();
        let ctx = Self {
            geng: geng.clone(),
            assets: assets.clone(),
            music: Rc::new(MusicManager::new()),
            options: Rc::new(RefCell::new(Options::default())),
        };
        ctx.force_set_options(options);
        Ok(ctx)
    }

    pub fn get_options(&self) -> Options {
        self.options.borrow().clone()
    }

    pub fn set_options(&self, options: Options) {
        let old = self.options.borrow();
        if *old != options {
            drop(old);
            self.force_set_options(options);
        }
    }

    fn force_set_options(&self, options: Options) {
        let mut old = self.options.borrow_mut();

        self.music
            .set_master_volume(options.master_volume * options.music_volume);

        preferences::save(crate::OPTIONS_STORAGE, &options);
        *old = options;
    }

    pub fn play_sfx(&self, sfx: &geng::Sound) {
        let options = self.get_options();
        let volume = options.master_volume * options.sfx_volume;
        let mut sfx = sfx.effect();
        sfx.set_volume(volume);
        sfx.play();
    }
}
