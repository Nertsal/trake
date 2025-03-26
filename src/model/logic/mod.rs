mod actions;
mod generation;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: FloatTime, player_input: PlayerInput) {
        self.real_time += delta_time;

        self.context
            .music
            .set_volume(self.train.train_speed.as_f32().clamp(0.0, 1.0));

        match self.phase {
            Phase::Setup => {}
            Phase::Resolution => {
                self.round_time += delta_time;
                self.move_train(delta_time, &player_input);
                self.collect_resources(delta_time);
                self.collide_train(delta_time);
            }
        }

        self.update_resources(delta_time);
        self.passive_particles(delta_time);
        self.process_particles(delta_time);
    }

    fn update_resources(&mut self, delta_time: FloatTime) {
        for resource in query!(self.items, (&mut resource.Get.Some)) {
            match &mut resource.state {
                ResourceNodeState::Spawning(time) => {
                    time.change(delta_time);
                    if time.is_max() {
                        resource.state = ResourceNodeState::Idle;
                    }
                }
                ResourceNodeState::Idle => {}
                ResourceNodeState::Despawning(time) => {
                    time.change(-delta_time);
                    if time.is_min() {
                        resource.state = ResourceNodeState::Idle;
                    }
                }
            }
        }
    }

    fn passive_particles(&mut self, _delta_time: FloatTime) {
        for wall in query!(self.items, (&wall.Get.Some)) {
            if wall.collider.check(&self.depo) {
                continue;
            }
            self.particles_queue.push(SpawnParticles {
                kind: ParticleKind::Wall,
                density: r32(0.5),
                distribution: ParticleDistribution::Aabb(wall.collider.compute_aabb()),
                size: r32(0.05)..=r32(0.1),
                ..default()
            });
        }
    }

    fn process_particles(&mut self, delta_time: FloatTime) {
        // Floating texts
        let mut dead_ids = Vec::new();
        for (id, position, velocity, lifetime) in query!(
            self.floating_texts,
            (id, &mut position, &velocity, &mut lifetime)
        ) {
            *position += *velocity * delta_time;
            lifetime.change(-delta_time);
            if lifetime.is_min() {
                dead_ids.push(id);
            }
        }
        for id in dead_ids {
            self.floating_texts.remove(id);
        }

        // Particles
        let mut dead_ids = Vec::new();
        for (id, position, velocity, lifetime) in query!(
            self.particles,
            (id, &mut position, &velocity, &mut lifetime)
        ) {
            *position += *velocity * delta_time;
            lifetime.change(-delta_time);
            if lifetime.is_min() {
                dead_ids.push(id);
            }
        }
        for id in dead_ids {
            self.particles.remove(id);
        }
        let spawn = self.particles_queue.drain(..).flat_map(spawn_particles);
        for particle in spawn {
            self.particles.insert(particle);
        }
    }

    fn collect_resources(&mut self, _delta_time: FloatTime) {
        // TODO
        let mut rng = thread_rng();

        // let mut collected = Vec::new();
        // for wagon in &self.train.blocks {
        //     let grid_pos = self.grid.world_to_grid(wagon.collider.position);
        //     for (res_id, &res_pos, _res) in
        //         query!(self.grid_items, (id, &position, &resource.Get.Some))
        //     {
        //         if grid_pos == res_pos {
        //             collected.push(res_id);
        //         }
        //     }
        // }

        // if !collected.is_empty() {
        //     self.add_wagon(TrainBlockKind::Wagon);
        //     self.context.play_sfx(&self.context.assets.sounds.clop2);
        // }
        // for id in collected {
        //     if let Some(item) = self.grid_items.remove(id) {
        //         if let Some(res) = item.resource {
        //             log::debug!("Collected: {:?}", res);
        //             let position = self.grid.grid_to_world(item.position);

        //             let mut plus_score = 0;
        //             let mut plus_money = 0;

        //             match res {
        //                 Resource::PlusCent => {
        //                     plus_score += self.round_score / 5;
        //                     plus_money += self.money / 10;
        //                 }
        //                 Resource::Coin => {
        //                     plus_money += rng.gen_range(8..=13);
        //                 }
        //                 Resource::GhostFuel => {
        //                     // TODO
        //                 }
        //                 _ => (),
        //             }

        //             if let Some(config) = self.config.resources.get(&res) {
        //                 plus_score += config.value;
        //             }

        //             self.round_score += plus_score;
        //             self.money += plus_money;

        //             self.particles_queue.push(SpawnParticles {
        //                 kind: ParticleKind::Collect(res),
        //                 density: r32(10.0),
        //                 distribution: ParticleDistribution::Circle {
        //                     center: position,
        //                     radius: r32(0.5),
        //                 },
        //                 velocity: vec2(0.0, 1.0).as_r32(),
        //                 ..default()
        //             });

        //             if plus_score != 0 {
        //                 self.floating_texts
        //                     .insert(spawn_text(format!("{:+}", plus_score), position));
        //             }
        //         }
        //     }
        // }
    }

    fn collide_train(&mut self, _delta_time: FloatTime) {
        let Some(head) = self.train.wagons.front_mut() else {
            return;
        };

        if head.collider.check(&self.depo) {
            // Ignore wall collisions, go to depo
            if !self.train.in_depo {
                self.next_round();
            }
            return;
        }
        self.train.in_depo = false;

        let mut collision = false;

        // Bounds
        let bounds = Collider::aabb_outline(self.map_bounds);
        if head.collider.check(&bounds) {
            collision = true;
        }

        // Walls
        if !collision {
            for wall in query!(self.items, (&wall.Get.Some)) {
                if head.collider.check(&wall.collider) {
                    collision = true;
                    break;
                }
            }
        }

        if collision {
            let block = self.train.wagons.pop_front().unwrap();
            // TODO: lose health
            // let plus_score =
            //     -(self.round_score as f32 * thread_rng().gen_range(0.15..=0.25)).ceil() as Score;
            // self.round_score += plus_score;
            // if plus_score != 0 {
            //     self.floating_texts.insert(spawn_text(
            //         format!("{:+}", plus_score),
            //         block.collider.position,
            //     ));
            // }

            self.particles_queue.push(SpawnParticles {
                kind: ParticleKind::WagonDestroyed,
                density: r32(20.0),
                distribution: ParticleDistribution::Circle {
                    center: block.collider.position,
                    radius: r32(0.5),
                },
                size: r32(0.1)..=r32(0.15),
                velocity: -block.collider.rotation.unit_vec()
                    * (self.train.train_speed * r32(0.5)).clamp(r32(0.5), r32(1.0)),
                ..default()
            });

            self.context.play_sfx(&self.context.assets.sounds.puff);
        }
    }

    fn add_wagon(&mut self, stats: WagonStats) {
        let Some(tail) = self.train.wagons.back() else {
            return;
        };

        let radius = |block: &Wagon| match block.collider.shape {
            Shape::Circle { radius } => radius,
            Shape::Rectangle { width, .. } => width * r32(0.5),
            Shape::RectangleOutline { width, .. } => width * r32(0.5),
        };

        let space = self.config.train.wagon_spacing + stats.size.x / r32(2.0) + radius(tail);
        let position = tail.collider.position - tail.collider.rotation.unit_vec() * space;
        let rotation = tail.collider.rotation;

        let mut wagon = Wagon::new(position, stats);
        wagon.collider.rotation = rotation;
        self.train.wagons.push_back(wagon);
    }

    fn move_train(&mut self, delta_time: FloatTime, player_input: &PlayerInput) {
        if self.train.wagons.is_empty() {
            self.next_round();
            return;
        }

        let move_head = |wagon: &mut Wagon, player_input: &PlayerInput| {
            // Turn by player input
            wagon.collider.rotation += self.config.train.turn_speed
                * player_input.turn
                * delta_time
                * self.train.train_speed.min(Coord::ONE);

            // Movement
            wagon.collider.position +=
                wagon.collider.rotation.unit_vec() * self.train.train_speed * delta_time;
        };

        let move_wagon = |head: &mut Wagon, wagon: &mut Wagon| {
            let radius = |block: &Wagon| match block.collider.shape {
                Shape::Circle { radius } => radius,
                Shape::Rectangle { width, .. } => width * r32(0.5),
                Shape::RectangleOutline { width, .. } => width * r32(0.5),
            };

            let space = self.config.train.wagon_spacing + radius(wagon);
            let to = head.collider.position - head.collider.rotation.unit_vec() * radius(head);
            let delta = to - wagon.collider.position;
            wagon.collider.position = to - delta.normalize_or_zero() * space;
            wagon.collider.rotation = delta.arg();
        };

        // Move wagons
        let mut blocks = self.train.wagons.iter_mut();
        if let Some(mut head) = blocks.next() {
            self.particles_queue.push(SpawnParticles {
                kind: ParticleKind::Steam,
                density: r32(4.0) * self.train.train_speed.clamp(r32(0.5), r32(5.0)),
                distribution: ParticleDistribution::Circle {
                    center: head.collider.position
                        + head.collider.rotation.unit_vec() * head.status.size.x / r32(2.5),
                    radius: r32(0.1),
                },
                size: r32(0.05)..=r32(0.15),
                velocity: -head.collider.rotation.unit_vec()
                    * (self.train.train_speed * r32(0.5)).clamp(r32(0.1), r32(0.5)),
                ..default()
            });
            move_head(head, player_input);

            for block in blocks {
                move_wagon(head, block);
                head = block;
            }
        }

        // Acceleration
        self.train.target_speed = self.config.train.speed;
        let target = self.train.target_speed;
        let current_speed = self.train.train_speed;
        let acceleration = if target > current_speed {
            self.config.train.acceleration
        } else {
            -self.config.train.deceleration
        };
        self.train.train_speed =
            current_speed + (acceleration * delta_time).clamp_abs((target - current_speed).abs());

        if self.train.train_speed == Coord::ZERO {
            self.next_round();
        }
    }
}

fn spawn_text(palette: &Palette, text: impl Into<Name>, position: vec2<Coord>) -> FloatingText {
    let mut rng = thread_rng();
    let text = text.into();

    let angle = Angle::from_radians(r32(rng.gen_range(1.0..=2.0)));
    let speed = r32(rng.gen_range(0.5..=1.0));
    let velocity = angle.unit_vec() * speed;

    FloatingText {
        position,
        velocity,
        size: r32(0.75),
        color: if text.starts_with('-') {
            palette.text_negative
        } else {
            palette.text_positive
        },
        lifetime: Bounded::new_max(r32(1.5)),
        text,
    }
}
