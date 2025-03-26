use super::*;

impl Model {
    pub fn init(&mut self) {
        // Camera
        self.camera.center = self.map_bounds.center().as_f32();

        // Walls
        self.items = default();

        self.next_round();
    }

    pub fn next_round(&mut self) {
        log::debug!("Round ended");
        self.round_time = FloatTime::ZERO;
        let mut rng = thread_rng();

        // Depo
        let size = self.config.depo_size;
        let (grid_min, grid_max) = (self.map_bounds.bottom_left(), self.map_bounds.top_right());
        let y = rng.gen_range(grid_min.y..=grid_max.y - size.y);
        self.depo = Collider::aabb(
            Aabb2::point(vec2(grid_min.x, y))
                .extend_left(size.x)
                .extend_up(size.y),
        );

        // Train
        let mut wagons = self.deck.wagons.clone().into_iter();
        self.train = Train {
            in_depo: true,
            target_speed: r32(0.0),
            train_speed: r32(0.0),
            wagons: wagons
                .next()
                .into_iter()
                .map(|stats| Wagon::new(self.depo.position, stats))
                .collect(),
        };
        for wagon in wagons {
            self.add_wagon(wagon);
        }

        // Cleanup
        let ids: Vec<_> = query!(self.items, (id, &wall))
            .filter(|(_, wall)| wall.is_none())
            .map(|(id, _)| id)
            .collect();
        for id in ids {
            self.items.remove(id);
        }

        // Spawn items
        let select_position = |rng: &mut ThreadRng,
                               radius: Coord,
                               items: &StructOf<Arena<Item>>|
         -> Option<vec2<Coord>> {
            let area = self.map_bounds.extend_uniform(-radius);
            for _ in 0..10 {
                let pos = vec2(
                    rng.gen_range(area.min.x..=area.max.x),
                    rng.gen_range(area.min.y..=area.max.y),
                );
                if query!(items, (&position)).any(|&other_pos| (pos - other_pos).len() < radius) {
                    continue;
                }
                return Some(pos);
            }
            None
        };

        for (&kind, config) in &self.config.resources {
            if let Some(position) = select_position(&mut rng, r32(0.5), &self.items) {
                let config = config.clone();
                self.items.insert(Item {
                    position,
                    resource: Some(ResourceNode {
                        kind,
                        data: config,
                        state: ResourceNodeState::Spawning(Bounded::new_zero(
                            self.config.resource.spawn_time,
                        )),
                    }),
                    wall: None,
                });
            }
        }

        // Shop
        let upgrades = 2;
        let options = [
            (Upgrade::Speed, 15),
            (Upgrade::Feather, 10),
            (Upgrade::Turning, 10),
        ];
        let discounts = [(0.0, 4.0), (0.1, 3.0), (0.25, 2.0), (0.50, 1.0)];
        let &(discount, _) = discounts.choose_weighted(&mut rng, |(_, w)| *w).unwrap();
        let discount_i = rng.gen_range(0..upgrades);
        let upgrades = options.choose_multiple(&mut rng, upgrades);
        self.shop = upgrades
            .enumerate()
            .map(|(i, (upgrade, mut price))| {
                if i == discount_i {
                    price -= (price as f32 * discount).ceil() as Money;
                }
                ShopItem {
                    upgrade: upgrade.clone(),
                    price,
                    can_purchase: true,
                }
            })
            .collect();

        self.phase = Phase::Setup;
    }
}
