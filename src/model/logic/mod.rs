mod actions;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: FloatTime) {
        self.move_train(delta_time);
        self.collect_resources(delta_time);
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
            self.context.play_sfx(&self.context.assets.sounds.choochoo);
        }
        for id in collected {
            if let Some(item) = self.grid_items.remove(id) {
                if let Some(res) = item.resource {
                    log::info!("Collected: {:?}", res);
                }
            }
        }
    }

    fn move_train(&mut self, delta_time: FloatTime) {
        if self.train.blocks.is_empty() {
            return;
        }

        // Returns whether the wagon is on a rail
        let move_head = |wagon: &mut TrainBlock| -> bool {
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
                let current_side = (offset.arg().normalized_2pi().as_degrees().as_f32() / 90.0)
                    .round() as usize
                    % 4;

                if vec2::dot(offset, move_dir) < Coord::ZERO {
                    // Entering the rail
                    wagon.entering_rail = true;
                } else {
                    // Leaving the rail
                    // Crossed the center of the rail - turn
                    if wagon.entering_rail && !cons[current_side] {
                        // Find the turn
                        if cons[(current_side + 1) % 4] {
                            // Turn left
                            wagon.collider.rotation += Angle::from_degrees(r32(90.0));
                            wagon.collider.position = rail_pos;
                            wagon.path.push_front(rail_pos);
                        } else if cons[(current_side + 3) % 4] {
                            // Turn right
                            wagon.collider.rotation -= Angle::from_degrees(r32(90.0));
                            wagon.collider.position = rail_pos;
                            wagon.path.push_front(rail_pos);
                        }
                    }
                    wagon.entering_rail = false;
                }

                cons[current_side]
            } else {
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
                if head.path.len() > i + 1 {
                    head.path.drain(i + 1..);
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
            if move_head(head) {
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
        let current_speed = self.train.train_speed;
        let acceleration = if self.train.target_speed > current_speed {
            self.config.train.acceleration
        } else {
            -self.config.train.deceleration
        };
        self.train.train_speed = current_speed
            + (acceleration * delta_time)
                .clamp_abs((self.train.target_speed - current_speed).abs());
    }
}
