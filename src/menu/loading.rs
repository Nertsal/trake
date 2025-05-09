use crate::{prelude::*, render::util::TextRenderOptions, task::Task, ui::layout::AreaOps};

pub struct LoadingScreen<T> {
    geng: Geng,
    assets: Rc<LoadingAssets>,
    options: Options,
    future: Option<Task<T>>,
    result: Option<T>,

    /// Fake load time so the screen doesnt flash
    min_load_time: f64,
    real_time: f64,
    texts: Vec<&'static str>,
    current_text: usize,
    text_timer: Bounded<f64>,
}

impl<T: 'static> LoadingScreen<T> {
    pub fn new(
        geng: &Geng,
        assets: Rc<LoadingAssets>,
        future: impl Future<Output = T> + 'static,
    ) -> Self {
        // let height = 360;
        // let size = vec2(height * 16 / 9, height);
        Self {
            geng: geng.clone(),
            assets,
            options: preferences::load(crate::OPTIONS_STORAGE).unwrap_or_default(),
            future: Some(Task::new(geng, future)),
            result: None,

            min_load_time: 0.0,
            real_time: 0.0,
            texts: vec![
                "Loading assets...",
                "Turning the lights on...",
                "Initializing evil... >:3",
                "Why is this taking so long?",
            ],
            current_text: 0,
            text_timer: Bounded::new_max(2.0),
        }
    }

    fn check_result(&mut self) -> Option<T> {
        // Poll future
        if let Some(task) = self.future.take() {
            match task.poll() {
                Ok(result) => {
                    self.result = Some(result);
                }
                Err(task) => {
                    self.future = Some(task);
                }
            }
        }

        // Check completion and timer
        if self.real_time > self.min_load_time {
            if let Some(result) = self.result.take() {
                return Some(result);
            }
        }

        None
    }

    pub async fn run(mut self) -> Option<T> {
        let geng = self.geng.clone();
        let mut timer = Timer::new();

        let mut events = geng.window().events();
        while let Some(event) = events.next().await {
            use geng::State;
            match event {
                geng::Event::Draw => {
                    let delta_time = timer.tick().as_secs_f64();
                    let delta_time = delta_time.min(0.05);
                    self.update(delta_time);

                    let window_size = geng.window().real_size();
                    if window_size.x != 0 && window_size.y != 0 {
                        geng.window().with_framebuffer(|framebuffer| {
                            self.draw(framebuffer);
                        });
                    }

                    if let Some(result) = self.check_result() {
                        return Some(result);
                    }
                }
                _ => self.handle_event(event),
            }
        }

        None
    }

    fn draw_text(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        options: TextRenderOptions,
    ) {
        let font = &self.assets.font;
        font.draw(framebuffer, camera, text, position, options);
    }
}

impl<T: 'static> geng::State for LoadingScreen<T> {
    fn update(&mut self, delta_time: f64) {
        self.real_time += delta_time;
        self.text_timer.change(-delta_time);
        if self.text_timer.is_min() {
            self.text_timer.set_ratio(1.0);
            self.current_text = (self.current_text + 1) % self.texts.len();
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.options.theme.dark), None, None);

        let framebuffer_size = framebuffer.size().as_f32();
        let font_size = framebuffer_size.y * 0.08;
        let theme = self.options.theme;

        let screen = Aabb2::ZERO.extend_positive(framebuffer_size);
        let camera = &geng::PixelPerfectCamera;

        // Background
        {
            let gif = &self.assets.background;
            let duration: f32 = gif.iter().map(|frame| frame.duration).sum();
            let mut time = (self.real_time as f32 / duration).fract() * duration;
            if let Some(frame) = gif.iter().find(|frame| {
                time -= frame.duration;
                time <= 0.0
            }) {
                self.geng.draw2d().textured_quad(
                    framebuffer,
                    camera,
                    screen,
                    &frame.texture,
                    Color::WHITE,
                );
            }
        }

        // Title
        let title = geng_utils::pixel::pixel_perfect_aabb(
            screen.align_pos(vec2(0.5, 0.8)),
            vec2(0.5, 0.5),
            self.assets.title.size() * 2 * (framebuffer.size().y / 360),
            camera,
            framebuffer_size,
        );
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            title,
            &self.assets.title,
            theme.light,
        );

        // Funny text
        if let Some(text) = self.texts.get(self.current_text) {
            let pos = screen.align_pos(vec2(0.5, 0.45));
            self.draw_text(
                framebuffer,
                camera,
                text,
                pos,
                TextRenderOptions::new(font_size).color(theme.light),
            );
        }
    }
}
