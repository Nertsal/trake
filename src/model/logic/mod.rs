mod actions;
mod generation;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: FloatTime, player_input: PlayerInput) {
        self.real_time += delta_time;

        match self.phase {
            Phase::Setup => {}
            Phase::Resolution => {
                self.round_time += delta_time;
                self.move_train(delta_time, &player_input);
                self.collect_resources(delta_time);
                self.collide_train(delta_time);
            }
        }
    }

    fn collect_resources(&mut self, _delta_time: FloatTime) {
        let mut collected = Vec::new();
        for wagon in &self.train.blocks {
            let grid_pos = self.grid.world_to_grid(wagon.collider.position);
            for (res_id, &res_pos, _res) in
                query!(self.grid_items, (id, &position, &resource.Get.Some))
            {
                if grid_pos == res_pos {
                    collected.push(res_id);
                }
            }
        }

        if !collected.is_empty() {
            self.add_wagon(TrainBlockKind::Wagon);
            self.context.play_sfx(&self.context.assets.sounds.choochoo);
        }
        for id in collected {
            if let Some(item) = self.grid_items.remove(id) {
                if let Some(res) = item.resource {
                    log::debug!("Collected: {:?}", res);

                    if let Resource::PlusCent = res {
                        self.round_score += self.round_score / 5;
                    }

                    if let Some(config) = self.config.resources.get(&res) {
                        self.round_score += config.value;
                    }
                }
            }
        }
    }

    fn collide_train(&mut self, _delta_time: FloatTime) {
        let Some(head) = self.train.blocks.front_mut() else {
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
        for wall in query!(self.grid_items, (&wall.Get.Some)) {
            if head.collider.check(&wall.collider) {
                collision = true;
                break;
            }
        }

        if collision {
            self.train.blocks.pop_front();
        }
    }

    fn add_wagon(&mut self, kind: TrainBlockKind) {
        let Some(tail) = self.train.blocks.back() else {
            return;
        };
        let mut space_left = self.config.train.wagon_spacing + self.config.train.wagon_size.x;
        let (anchor, dir) = if let Some((to, from)) = std::iter::once(tail.collider.position)
            .chain(tail.path.iter().copied())
            .tuple_windows()
            .find(|(to, from)| {
                let dist = (*to - *from).len();
                if space_left <= dist {
                    true
                } else {
                    space_left -= dist;
                    false
                }
            }) {
            (to, (from - to).normalize_or_zero())
        } else {
            space_left = self.config.train.wagon_spacing + self.config.train.wagon_size.x;
            (tail.collider.position, -tail.collider.rotation.unit_vec())
        };
        let position = anchor + dir * space_left;
        let rotation = (-dir).arg();
        self.train.blocks.push_back(TrainBlock {
            kind,
            collider: Collider {
                shape: Shape::rectangle(self.config.train.wagon_size),
                position,
                rotation,
            },
            snapped_to_rail: false,
            entering_rail: false,
            path: VecDeque::new(),
        });
    }

    fn move_train(&mut self, delta_time: FloatTime, player_input: &PlayerInput) {
        if self.train.blocks.is_empty() {
            return;
        }

        // Returns whether the wagon is on a rail
        let move_head = |wagon: &mut TrainBlock, player_input: &PlayerInput| -> bool {
            let move_dir = wagon.collider.rotation.unit_vec();
            let pos = self.grid.world_to_grid(wagon.collider.position);
            let on_rail = if let Some((_rail_pos, rail)) =
                query!(self.grid_items, (&position, &rail.Get.Some))
                    .find(|(&position, _)| position == pos)
            {
                // On a rail
                let cons = Connections::from(rail.orientation);
                let cons = [cons.right, cons.top, cons.left, cons.bottom];

                let rail_pos = self.grid.grid_to_world(pos);
                let offset = wagon.collider.position - rail_pos;

                let face_side = (wagon
                    .collider
                    .rotation
                    .normalized_2pi()
                    .as_degrees()
                    .as_f32()
                    / 90.0)
                    .round() as usize
                    % 4;
                let back_side = (face_side + 2) % 4;

                let ninety = Angle::from_degrees(r32(90.0));
                if cons[back_side] && vec2::dot(offset, move_dir) < Coord::ZERO {
                    // Entering the rail
                    // Align train with the rail
                    wagon.collider.rotation = ninety * r32(face_side as f32);
                    let rail_dir = wagon.collider.rotation.unit_vec();
                    wagon.collider.position = rail_pos
                        + rail_dir * vec2::dot(wagon.collider.position - rail_pos, rail_dir);

                    if !wagon.entering_rail {
                        // Just entered
                        wagon.path.push_front(wagon.collider.position);
                    }

                    wagon.snapped_to_rail = true;
                    wagon.entering_rail = true;
                    true
                } else {
                    // Leaving the rail
                    let rail_dir = ninety * r32(face_side as f32);

                    // Crossed the center of the rail - turn
                    let on_rail =
                        if wagon.snapped_to_rail && wagon.entering_rail && !cons[face_side] {
                            // Find the turn
                            if cons[(face_side + 1) % 4] {
                                // Turn left
                                wagon.collider.rotation = rail_dir + ninety;
                                wagon.collider.position = rail_pos;
                                wagon.path.push_front(rail_pos);
                                true
                            } else if cons[(face_side + 3) % 4] {
                                // Turn right
                                wagon.collider.rotation = rail_dir - ninety;
                                wagon.collider.position = rail_pos;
                                wagon.path.push_front(rail_pos);
                                true
                            } else {
                                false
                            }
                        } else if cons[face_side] {
                            // Align train with the rail
                            wagon.collider.rotation = ninety * r32(face_side as f32);
                            let rail_dir = wagon.collider.rotation.unit_vec();
                            wagon.collider.position = rail_pos
                                + rail_dir
                                    * vec2::dot(wagon.collider.position - rail_pos, rail_dir);
                            if wagon.entering_rail {
                                wagon.path.push_front(wagon.collider.position);
                            }
                            true
                        } else {
                            false
                        };

                    wagon.snapped_to_rail = on_rail;
                    wagon.entering_rail = false;
                    on_rail
                }
            } else {
                // Turn by player input
                wagon.collider.rotation += self.config.train.turn_speed
                    * player_input.turn
                    * delta_time
                    * self.train.train_speed.min(Coord::ONE);

                false
            };

            // Movement
            wagon.collider.position +=
                wagon.collider.rotation.unit_vec() * self.train.train_speed * delta_time;

            on_rail
        };

        // Returns whether the wagon is on a rail
        let move_wagon = |head: &mut TrainBlock, wagon: &mut TrainBlock| -> bool {
            let move_on = |from: vec2<Coord>,
                           to: vec2<Coord>,
                           space: Coord,
                           wagon: &mut TrainBlock|
             -> bool {
                let delta = to - from;
                wagon.collider.position = to - delta.normalize_or_zero() * space;
                let new_rotation = delta.arg();
                if wagon.collider.rotation != new_rotation {
                    wagon.collider.rotation = new_rotation;
                    wagon.path.push_front(from);
                }

                let pos = self.grid.world_to_grid(wagon.collider.position);
                if let Some((_rail_pos, rail)) =
                    query!(self.grid_items, (&position, &rail.Get.Some))
                        .find(|(&position, _)| position == pos)
                {
                    let cons = Connections::from(rail.orientation);
                    let cons = [cons.right, cons.top, cons.left, cons.bottom];

                    let rail_pos = self.grid.grid_to_world(pos);
                    let offset = wagon.collider.position - rail_pos;
                    let current_side = (offset.arg().normalized_2pi().as_degrees().as_f32() / 90.0)
                        .round() as usize
                        % 4;

                    cons[current_side]
                } else {
                    false
                }
            };

            let mut space_left = self.config.train.wagon_spacing + self.config.train.wagon_size.x;
            if let Some((i, (to, from))) = std::iter::once(head.collider.position)
                .chain(head.path.iter().copied())
                .chain(std::iter::once(wagon.collider.position))
                .tuple_windows()
                .enumerate()
                .find(|(_, (to, from))| {
                    let dist = (*to - *from).len();
                    if space_left <= dist {
                        true
                    } else {
                        space_left -= dist;
                        false
                    }
                })
            {
                if head.path.len() > i {
                    head.path.drain(i..);
                }
                move_on(from, to, space_left, wagon)
            } else {
                let from = wagon.collider.position;
                let to = head.path.back().copied().unwrap_or(head.collider.position);
                move_on(from, to, space_left + (to - from).len(), wagon)
            }
        };

        // Move wagons
        let mut on_rail = 0;
        let mut blocks = self.train.blocks.iter_mut();
        if let Some(mut head) = blocks.next() {
            if move_head(head, player_input) {
                on_rail += 1;
            }

            for block in blocks {
                if move_wagon(head, block) {
                    on_rail += 1;
                }
                head = block;
            }
        }

        // Acceleration
        self.train.target_speed = self.config.train.offrail_speed
            + (self.config.train.rail_speed - self.config.train.offrail_speed)
                * r32(on_rail as f32 / self.train.blocks.len() as f32);
        let slowdown_s = self.config.train.offrail_speed;
        let slowdown_t = slowdown_s / self.config.train.overtime_slowdown;
        let t = self.round_time / slowdown_t;
        let slowdown = t * t * t * slowdown_s;
        let target = (self.train.target_speed - slowdown).max(Coord::ZERO);
        let current_speed = self.train.train_speed;
        let acceleration = if target > current_speed {
            self.config.train.acceleration
        } else {
            -self.config.train.deceleration
        };
        self.train.train_speed =
            current_speed + (acceleration * delta_time).clamp_abs((target - current_speed).abs());
    }
}
