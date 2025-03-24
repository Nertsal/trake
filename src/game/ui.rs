use super::*;

use crate::ui::{layout::AreaOps, widget::*};

pub struct GameUi {
    pub game: WidgetState,
}

impl GameUi {
    pub fn new() -> Self {
        Self {
            game: WidgetState::new(),
        }
    }

    pub fn layout(
        &mut self,
        model: &Model,
        screen: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Vec<GameAction> {
        let mut actions = Vec::new();

        let atlas = &context.context.assets.atlas;
        let mut screen = screen.fit_aabb(vec2(16.0, 9.0), vec2(0.5, 0.5));
        let font_size = screen.height() * 0.04;
        let layout_size = screen.height() * 0.03;

        context.font_size = font_size;
        context.layout_size = layout_size;
        context.screen = screen;

        let game_size = crate::GAME_RESOLUTION * 3;
        let mut game = screen.cut_right(game_size.x as f32);
        let excess = game.height() - game_size.y as f32;
        game.cut_top(excess / 2.0);
        game.cut_bottom(excess / 2.0);
        let mut left_bar = screen;

        self.game.update(game, context);

        // Left bar
        {
            let title = left_bar.cut_top(font_size * 2.0);

            let pos = left_bar.cut_top(font_size * 1.0);
            let score = context.state.get_root_or(|| TextWidget::new("Quota"));
            score.update(pos, context);
            score.text = format!("Quota: {}/{}", model.quota_score, model.current_quota).into();

            let pos = left_bar.cut_top(font_size * 1.0);
            let score = context.state.get_root_or(|| TextWidget::new("Score"));
            score.update(pos, context);
            score.text = format!("Score: {}", model.round_score).into();

            let mut right = left_bar
                .cut_top(font_size * 1.0)
                .with_width(font_size * 3.0, 0.5);
            let left = right.split_left(0.5);
            context
                .state
                .get_root_or(|| IconWidget::new(atlas.coin()))
                .update(left, context);
            let money = context.state.get_root_or(|| TextWidget::new(""));
            money.update(right, context);
            money.text = format!("{}", model.money).into();

            left_bar.cut_top(font_size);

            let pos = left_bar
                .cut_top(font_size * 1.2)
                .with_width(font_size * 4.0, 0.5);
            let launch = context.state.get_root_or(|| ButtonWidget::new("Launch"));
            launch.update(pos, context);
            if launch.text.state.clicked {
                actions.push(GameAction::LaunchTrain);
            }
        }

        // Shop
        {
            let mut shop = left_bar.cut_bottom(font_size * 6.0);

            shop.cut_left(font_size * 4.0);
            shop.cut_bottom(font_size * 2.0);

            let title = shop.cut_top(font_size * 1.2);
            let text = context
                .state
                .get_root_or(|| TextWidget::new("Shop").aligned(vec2(0.0, 1.0)));
            text.update(title, context);

            for (i, item) in model.shop.iter().enumerate() {
                let mut pos = shop.cut_left(font_size * 3.0);

                let mut price = pos.cut_bottom(font_size);
                let icon = price.split_left(0.5);
                context
                    .state
                    .get_root_or(|| IconWidget::new(atlas.coin()))
                    .update(icon, context);
                let text = context
                    .state
                    .get_root_or(|| TextWidget::new("").aligned(vec2(0.0, 0.5)));
                text.update(price, context);
                text.text = format!("{}", item.price).into();

                let pos = pos.extend_uniform(-font_size * 0.1);
                let button = context
                    .state
                    .get_root_or(|| IconButtonWidget::new_normal(atlas.circle()));
                button.icon.texture = match item.upgrade {
                    Upgrade::Resource(resource) => match resource {
                        Resource::PlusCent => atlas.plus_cent(),
                        Resource::GhostFuel => atlas.ghost_fuel(),
                        _ => unimplemented!(),
                    },
                    Upgrade::Speed => atlas.speed(),
                    Upgrade::Feather => atlas.feather(),
                    Upgrade::Turning => atlas.spiral(),
                };
                button.update(pos, context);
                if button.state.clicked {
                    actions.push(GameAction::BuyShop(i));
                }
            }
        }

        actions
    }
}
