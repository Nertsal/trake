use super::*;

impl Model {
    pub fn init(&mut self) {
        // Camera
        self.camera.center = self
            .grid
            .grid_to_world(self.config.map_size / 2 + vec2(1, 1))
            .as_f32();

        // Walls
        self.grid_items = default();
        let mut set_wall_at = |position: vec2<ICoord>| {
            self.grid_items.insert(GridItem {
                position,
                rail: None,
                resource: None,
                wall: Some(Wall {
                    collider: Collider::aabb(
                        Aabb2::point(self.grid.grid_to_world(position))
                            .extend_symmetric(self.grid.cell_size * r32(0.9 / 2.0)),
                    ),
                }),
            });
        };
        for x in 0..=self.config.map_size.x + 1 {
            set_wall_at(vec2(x, 0));
            set_wall_at(vec2(x, self.config.map_size.y + 1));
        }
        for y in 1..=self.config.map_size.y {
            set_wall_at(vec2(0, y));
            set_wall_at(vec2(self.config.map_size.x + 1, y));
        }

        self.next_round();
    }

    pub fn next_round(&mut self) {
        log::debug!("Round ended");
        let mut rng = thread_rng();

        // Score
        self.total_score += self.round_score;
        self.quota_score += self.round_score;
        self.round_score = 0;
        if self.quota_score >= self.current_quota {
            // Next quota
            self.quotas_completed += 1;
            let noise = rng.gen_range(0.9..=1.1);
            self.current_quota +=
                (100.0 * (self.quotas_completed.sqr() as f32 / 16.0) * noise) as Score;
        } else if self.quota_day >= 2 {
            // Quota failed
            todo!("you failed");
        } else {
            // Next day
            self.quota_day += 1;
        }

        // Depo
        let size = self.config.depo_size;
        let grid_min = self.grid.gridf_to_world(vec2(0.5, 0.5).as_r32());
        let grid_max = self
            .grid
            .gridf_to_world(self.config.map_size.map(|x| x as f32 - 0.5).as_r32());
        let y = rng.gen_range(grid_min.y..=grid_max.y - size.y);
        self.depo = Collider::aabb(
            Aabb2::point(vec2(grid_min.x, y))
                .extend_left(size.x)
                .extend_up(size.y),
        );

        // Train
        self.train = Train {
            in_depo: true,
            target_speed: r32(0.0),
            train_speed: r32(0.0),
            blocks: vec![TrainBlock::new_locomotive(
                &self.config.train,
                self.depo.position,
            )]
            .into(),
        };

        // Cleanup
        let ids: Vec<_> = query!(self.grid_items, (id, &wall))
            .filter(|(_, wall)| wall.is_none())
            .map(|(id, _)| id)
            .collect();
        for id in ids {
            self.grid_items.remove(id);
        }

        // Spawn items
        let mut positions: Vec<_> = (1..=self.config.map_size.x)
            .flat_map(|x| (1..=self.config.map_size.y).map(move |y| vec2(x, y)))
            .collect();
        positions.shuffle(&mut rng);

        for &res in &self.deck.resources {
            if let Some(position) = positions.pop() {
                self.grid_items.insert(GridItem {
                    position,
                    rail: None,
                    resource: Some(res),
                    wall: None,
                });
            }
        }
        for &kind in &self.deck.rails {
            let orientation = RailOrientation {
                kind,
                rotation: rng.gen_range(0..=3),
            };
            if let Some(position) = positions.pop() {
                self.grid_items.insert(GridItem {
                    position,
                    rail: Some(Rail { orientation }),
                    resource: None,
                    wall: None,
                });
            }
        }

        self.phase = Phase::Setup;
    }
}
