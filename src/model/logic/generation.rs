use super::*;

impl Model {
    pub fn init(&mut self) {
        // Camera
        self.camera.center = self.map_bounds.center().as_f32();

        // Walls
        self.items = default();

        self.generate_map(vec![TunnelEffect::Snow]);
    }

    pub fn generate_map(&mut self, effects: Vec<TunnelEffect>) {
        log::debug!("Generating the map...");
        self.round_simulation_time = FloatTime::ZERO;
        let mut rng = thread_rng();

        // Effects
        self.game_time_scale = FloatTime::ONE;
        let mut wind_strength = Coord::ZERO;
        let mut snow = 0;
        let mut rocks = 0;
        for effect in effects {
            match effect {
                TunnelEffect::TimeWarp { time_scale } => {
                    self.game_time_scale *= time_scale;
                }
                TunnelEffect::Wind { strength } => {
                    wind_strength += strength;
                }
                TunnelEffect::Snow => {
                    snow += 2;
                }
                TunnelEffect::Rocks => {
                    rocks += 2;
                }
                _ => (),
            }
        }
        self.wind = Angle::from_degrees(r32(rng.gen_range(0.0..=360.0))).unit_vec() * wind_strength;

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
            fuel: Fuel::ZERO,
            head_damage: self.config.train.head_damage,
            wagons: wagons
                .next()
                .into_iter()
                .map(|stats| Wagon::new(self.depo.position, stats))
                .collect(),
        };
        for wagon in wagons {
            self.add_wagon(wagon);
        }
        self.train.fuel = self.train.fuel_capacity();

        // Tunnels
        let combine_presets = |prefix: &TunnelPreset, suffix: &TunnelPreset| {
            let mut effects = prefix.effects.clone();
            effects.extend(suffix.effects.iter().cloned());
            Tunnel {
                collider: Collider::circle(vec2::ZERO, Coord::ZERO),
                name: format!("{} {} tunnel", prefix.name, suffix.name).into(),
                effects,
            }
        };

        let tunnels = (0..3).map(|_| {
            combine_presets(
                self.config.tunnels.prefix.choose(&mut rng).unwrap(),
                self.config.tunnels.suffix.choose(&mut rng).unwrap(),
            )
        });
        let n = tunnels.len();
        self.tunnels = tunnels
            .enumerate()
            .map(|(i, tunnel)| {
                let t = (i as f32 + 1.0) / (n as f32 + 1.0);
                let position = self.map_bounds.align_pos(vec2(t, 1.0).as_r32());
                let size = vec2(2.0, 4.0).as_r32();
                Tunnel {
                    collider: Collider::aabb(
                        Aabb2::point(position)
                            .extend_symmetric(vec2(size.x, Coord::ZERO) / r32(2.0))
                            .extend_up(size.y),
                    ),
                    ..tunnel
                }
            })
            .collect();

        // Cleanup
        self.items = default();
        self.entities = default();

        // Spawn resource nodes
        for (&kind, config) in &self.config.resources {
            if let Some(position) = select_position(
                &mut rng,
                self.map_bounds,
                r32(0.5),
                &self.items,
                &self.entities,
            ) {
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

        // Spawn rocks
        for _ in 0..rocks {
            let radius = r32(1.0);
            if let Some(position) = select_position(
                &mut rng,
                self.map_bounds,
                radius,
                &self.items,
                &self.entities,
            ) {
                self.entities.insert(Entity {
                    collider: Collider::circle(position, radius),
                    velocity: vec2::ZERO,
                    health: Some(Bounded::new_max(r32(0.1))),
                    team: None,
                    damage_on_collision: Some(r32(3.0)),
                    ai: None,
                    snow: None,
                });
            }
        }

        // Spawn snow
        for _ in 0..snow {
            let radius = r32(0.8);
            let area = self.map_bounds.extend_uniform(-radius);
            let position = vec2(
                rng.gen_range(area.min.x..=area.max.x),
                rng.gen_range(area.min.y..=area.max.y),
            );
            self.entities.insert(Entity {
                collider: Collider::circle(position, radius),
                velocity: vec2::ZERO,
                health: None,
                team: None,
                damage_on_collision: None,
                ai: None,
                snow: Some(()),
            });
        }

        // Spawn enemies
        for _ in 0..2 {
            if let Some(position) = select_position(
                &mut rng,
                self.map_bounds,
                r32(1.0),
                &self.items,
                &self.entities,
            ) {
                self.entities.insert(Entity {
                    collider: Collider::circle(position, r32(0.3)),
                    velocity: vec2::ZERO,
                    health: Some(Bounded::new_max(r32(10.0))),
                    team: Some(Team::Enemy),
                    damage_on_collision: None,
                    ai: Some(EntityAi::Shooter(ShooterAi {
                        range: r32(2.0),
                        shooting_speed: r32(0.5),
                        cooldown: Bounded::new_max(R32::ONE),
                        bullet_speed: r32(5.0),
                        bullet_damage: r32(4.0),
                    })),
                    snow: None,
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

        self.phase = Phase::Starting;
    }
}

pub fn select_position(
    rng: &mut ThreadRng,
    map_bounds: Aabb2<Coord>,
    radius: Coord,
    items: &StructOf<Arena<Item>>,
    entities: &StructOf<Arena<Entity>>,
) -> Option<vec2<Coord>> {
    let area = map_bounds.extend_uniform(-radius);
    for _ in 0..10 {
        let pos = vec2(
            rng.gen_range(area.min.x..=area.max.x),
            rng.gen_range(area.min.y..=area.max.y),
        );
        if query!(items, (&position)).any(|&other_pos| (pos - other_pos).len() < radius) {
            continue;
        }
        if query!(entities, (&collider)).any(|coll| (pos - coll.position).len() < radius) {
            continue;
        }
        return Some(pos);
    }
    None
}
