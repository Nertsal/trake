mod actions;
mod generation;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: FloatTime, player_input: PlayerInput) {
        self.real_time += delta_time;
        let sim_delta_time = delta_time * self.game_time_scale;
        self.simulation_time += sim_delta_time;

        self.context
            .music
            .set_volume(self.train.train_speed.as_f32().clamp(0.0, 1.0));

        match self.phase {
            Phase::Starting => {}
            Phase::Action => {
                self.round_simulation_time += sim_delta_time;
                self.move_train(sim_delta_time, &player_input);
                self.collect_resources(sim_delta_time);
                self.collide_train(sim_delta_time);
            }
            Phase::Finished => {}
            Phase::Leaving { tunnel } => {
                self.check_tunnel(tunnel, sim_delta_time);
            }
        }

        self.update_entities(sim_delta_time);
        self.update_train(sim_delta_time);
        self.update_resources(sim_delta_time);
        self.passive_particles(sim_delta_time);
        self.process_particles(sim_delta_time);
    }

    fn game_over(&mut self) {
        todo!("you failed");
    }

    fn end_action(&mut self) {
        let Phase::Action = self.phase else { return };

        if self.train.wagons.is_empty() {
            self.game_over();
            return;
        }

        self.phase = Phase::Finished;
    }

    fn next_map(&mut self, tunnel: Tunnel) {
        self.generate_map(tunnel.effects);
    }

    fn check_tunnel(&mut self, tunnel: usize, _delta_time: FloatTime) {
        let Some(tunnel) = self.tunnels.get(tunnel) else {
            return;
        };

        if let Some(wagon) = self.train.wagons.front() {
            if wagon.collider.check(&tunnel.collider) {
                self.next_map(tunnel.clone());
            }
        }
    }

    fn update_entities(&mut self, delta_time: FloatTime) {
        let mut rng = thread_rng();

        // Ai/Control
        let mut spawns = Vec::new();
        for (collider, ai, team) in query!(self.entities, (&collider, &mut ai.Get.Some, &team)) {
            match ai {
                EntityAi::Shooter(ai) => {
                    // Update cooldown
                    ai.cooldown.change(-ai.shooting_speed * delta_time);
                    if ai.cooldown.is_above_min() {
                        continue;
                    }

                    // Find a target
                    let target = match team {
                        Some(Team::Enemy) => self
                            .train
                            .wagons
                            .iter()
                            .find(|wagon| {
                                (collider.position - wagon.collider.position).len() <= ai.range
                            })
                            .map(|wagon| wagon.collider.position),
                        Some(Team::Player) => None,
                        None => None,
                    };
                    if let Some(target) = target {
                        ai.cooldown.set_ratio(R32::ONE);
                        spawns.push(Entity {
                            collider: Collider::circle(collider.position, r32(0.2)),
                            velocity: (target - collider.position).normalize_or_zero()
                                * ai.bullet_speed,
                            health: Some(Bounded::new_max(r32(0.1))),
                            team: *team,
                            damage_on_collision: Some(ai.bullet_damage),
                            ai: None,
                        });
                    }
                }
            }
        }

        // Spawn
        for spawn in spawns {
            self.entities.insert(spawn);
        }

        // Movement
        for (collider, &velocity) in query!(self.entities, (&mut collider, &velocity)) {
            collider.position += velocity * delta_time;
        }

        // Collisions
        for (team, collider, health, collision_damage) in query!(
            self.entities,
            (&team, &collider, &mut health, &damage_on_collision)
        ) {
            if let Some(Team::Enemy) = team {
                // Collide with train
                for wagon in &mut self.train.wagons {
                    if let Some(collision) = wagon.collider.collide(collider) {
                        if let &Some(damage) = collision_damage {
                            wagon.status.health.change(-damage);
                        }
                        if let Some(health) = health {
                            health.change(-self.train.head_damage);
                        }
                        self.particles_queue.push(SpawnParticles {
                            kind: ParticleKind::WagonDamaged,
                            density: r32(10.0),
                            distribution: ParticleDistribution::Circle {
                                center: collision.point,
                                radius: r32(0.4),
                            },
                            velocity: vec2(0.0, 1.0).as_r32(),
                            ..default()
                        });
                        break;
                    }
                }
            }
        }

        // Check health
        let mut remove = Vec::new();
        for (id, health) in query!(self.entities, (id, &health.Get.Some)) {
            if health.is_min() {
                remove.push(id);
            }
        }
        for id in remove {
            self.entities.remove(id);
        }
    }

    fn update_train(&mut self, _delta_time: FloatTime) {
        // Remove dead eagons
        self.train
            .wagons
            .retain(|wagon| wagon.status.health.is_above_min());
    }

    fn update_resources(&mut self, delta_time: FloatTime) {
        let mut rng = thread_rng();

        let mut remove = Vec::new();
        for (id, resource) in query!(self.items, (id, &mut resource.Get.Some)) {
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
                        remove.push(id);
                    }
                }
            }
        }

        for id in remove {
            let item = self.items.remove(id).unwrap();
            let resource = item.resource.unwrap();
            if let Some(data) = self.config.resources.get(&resource.kind).cloned() {
                if let Some(position) =
                    generation::select_position(&mut rng, self.map_bounds, r32(0.5), &self.items)
                {
                    self.items.insert(Item {
                        position,
                        resource: Some(ResourceNode {
                            kind: resource.kind,
                            data,
                            state: ResourceNodeState::Spawning(Bounded::new_zero(
                                self.config.resource.spawn_time,
                            )),
                        }),
                        wall: None,
                    });
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

    fn collect_resources(&mut self, delta_time: FloatTime) {
        let mut rng = thread_rng();

        let mut collect_sfx = false;
        for wagon in &mut self.train.wagons {
            let Some(collect) = &mut wagon.status.collect else {
                continue;
            };

            let wagon_collected = collect.total_stored();
            if wagon_collected >= collect.stats.capacity {
                // Cannot collect anymore resources
                collect.collecting = None;
                continue;
            }

            'collect: {
                if let Some(collecting) = &mut collect.collecting {
                    let Some((&position, resource)) = get!(
                        self.items,
                        collecting.resource,
                        (&position, &mut resource.Get.Some)
                    ) else {
                        // Resource no longer exists? stop i guess
                        collect.collecting = None;
                        break 'collect;
                    };

                    // Collection particles
                    let delta = position - wagon.collider.position;
                    self.particles_queue.push(SpawnParticles {
                        kind: ParticleKind::WagonDestroyed,
                        density: r32(2.0),
                        distribution: ParticleDistribution::Circle {
                            center: wagon.collider.position,
                            radius: r32(0.2),
                        },
                        velocity: delta / r32(0.4),
                        lifetime: r32(0.4)..=r32(0.5),
                        ..default()
                    });

                    // Advance collection progress
                    collecting
                        .completion
                        .change(resource.data.speed * collect.stats.speed * delta_time);
                    if collecting.completion.is_max() {
                        // Collect
                        collecting.completion.set_ratio(R32::ZERO);
                        let amount = resource
                            .data
                            .amount
                            .min(resource.data.per_collection)
                            .min(collect.stats.capacity - wagon_collected);
                        resource.data.amount -= amount;
                        if resource.data.amount <= 0 {
                            resource.state = ResourceNodeState::Despawning(Bounded::new_max(
                                self.config.resource.spawn_time,
                            ));
                        }
                        collect_sfx = true;
                        collect.collecting = None; // Search again in case there is another one closer

                        // Transfer to wagon storage
                        *collect.storage.entry(resource.kind).or_default() += amount;

                        // Particles
                        self.particles_queue.push(SpawnParticles {
                            kind: ParticleKind::Collect(resource.kind),
                            density: r32(10.0),
                            distribution: ParticleDistribution::Circle {
                                center: position,
                                radius: r32(0.3),
                            },
                            velocity: vec2(0.0, 1.0).as_r32(),
                            ..default()
                        });
                    }

                    // Check that the resource is still in range
                    if (wagon.collider.position - position).len() > collect.stats.range {
                        // Out of range - stop collecting
                        collect.collecting = None;
                    }
                }
            }

            // Not in an `else` branch because we want to switch resources same frame
            // when going out of range of the previous one
            if collect.collecting.is_none() {
                // Look for resources to collect
                let in_range = query!(self.items, (id, &position, &resource.Get.Some)).filter(
                    |(_, &pos, resource)| {
                        matches!(resource.state, ResourceNodeState::Idle)
                            && resource.kind == collect.stats.resource
                            && (wagon.collider.position - pos).len() <= collect.stats.range
                    },
                );
                if let Some((id, _, _)) = in_range.choose(&mut rng) {
                    collect.collecting = Some(WagonCollecting {
                        resource: id,
                        completion: Bounded::new_zero(R32::ONE),
                    });
                }
            }
        }

        if collect_sfx {
            self.context.play_sfx(&self.context.assets.sounds.clop2);
        }
    }

    fn collide_train(&mut self, _delta_time: FloatTime) {
        let Some(head) = self.train.wagons.front_mut() else {
            return;
        };

        if head.collider.check(&self.depo) {
            // Ignore wall collisions, go to depo
            if !self.train.in_depo {
                // TODO: idk
                self.generate_map(vec![]);
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
            self.end_action();
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
            if self.train.fuel > Fuel::ZERO {
                self.particles_queue.push(SpawnParticles {
                    kind: ParticleKind::Steam,
                    density: r32(4.0) * self.train.fuel.clamp(r32(0.5), r32(5.0)),
                    distribution: ParticleDistribution::Circle {
                        center: head.collider.position
                            + head.collider.rotation.unit_vec() * head.status.size.x / r32(2.5),
                        radius: r32(0.1),
                    },
                    size: r32(0.05)..=r32(0.15),
                    velocity: -head.collider.rotation.unit_vec()
                        * (self.train.train_speed * r32(0.5)).clamp(r32(0.1), r32(0.5))
                        + vec2(0.0, 0.5).as_r32(),
                    ..default()
                });
            }
            move_head(head, player_input);

            for block in blocks {
                move_wagon(head, block);
                head = block;
            }
        }

        // Acceleration
        self.train.target_speed = if self.train.fuel > Fuel::ZERO {
            self.config.train.speed
        } else {
            Coord::ZERO
        };
        let target = self.train.target_speed;
        let current_speed = self.train.train_speed;
        let acceleration = if target > current_speed {
            self.config.train.acceleration
        } else {
            self.config.train.deceleration
        };
        self.train.train_speed =
            current_speed + (target - current_speed).clamp_abs(acceleration * delta_time);

        // Spend fuel
        self.train.fuel = (self.train.fuel - self.config.train.fuel_consumption * delta_time)
            .clamp(Fuel::ZERO, self.train.fuel_capacity());

        // If the train stops, the round ends
        if self.train.train_speed == Coord::ZERO {
            self.end_action();
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
