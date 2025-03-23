mod actions;

use super::*;

impl Model {
    pub fn update(&mut self, delta_time: FloatTime) {
        if let Some(head) = self.train.blocks.front_mut() {
            let pos = self.grid.world_to_grid(head.collider.position);
            if let Some(rail) = self.rails.iter().find(|rail| rail.position == pos) {
                // On a rail
                let cons = Connections::from(rail.orientation);
                let cons = [cons.right, cons.top, cons.left, cons.bottom];

                let rail_pos = self.grid.grid_to_world(pos);
                let offset = head.collider.position - rail_pos;
                let current_side = (offset.arg().normalized_2pi().as_degrees().as_f32() / 90.0)
                    .round() as usize
                    % 4;
                // If leaving the rail, assume all effects
                // (like turning) have been completed
                if vec2::dot(offset, self.train.head_velocity) < Coord::ZERO {
                    // Entering the rail
                    let opp_side = (current_side + 2) % 4;
                    if cons[current_side] {
                        // Entered from a connected side
                        // If move past the center, check rail turn
                        let change = offset * (offset + self.train.head_velocity * delta_time);
                        if change.x < Coord::ZERO || change.y < Coord::ZERO {
                            if cons[opp_side] {
                                // Continue forward
                                self.train.target_speed = self.config.train.rail_speed;
                            } else {
                                // Find the turn
                                if cons[(opp_side + 1) % 4] {
                                    // Turn left
                                    self.train.head_velocity = self.train.head_velocity.rotate_90();
                                    head.collider.position = rail_pos;
                                    self.train.target_speed = self.config.train.rail_speed;
                                } else if cons[(opp_side + 3) % 4] {
                                    // Turn right
                                    self.train.head_velocity =
                                        -self.train.head_velocity.rotate_90();
                                    head.collider.position = rail_pos;
                                    self.train.target_speed = self.config.train.rail_speed;
                                } else {
                                    // No turn found - continue forward
                                    self.train.target_speed = self.config.train.offrail_speed;
                                }
                            }
                        }
                    } else {
                        // From a disconnected side - continue forward
                        // TODO: check speed
                        if cons[opp_side] {
                            self.train.target_speed = self.config.train.rail_speed;
                        } else {
                            self.train.target_speed = self.config.train.offrail_speed;
                        }
                    }
                }
            } else {
                self.train.target_speed = self.config.train.offrail_speed;
            }

            // Acceleration
            let current_speed = self.train.head_velocity.len();
            let acceleration = if self.train.target_speed > current_speed {
                self.config.train.acceleration
            } else {
                -self.config.train.deceleration
            };
            let speed = current_speed
                + (acceleration * delta_time)
                    .clamp_abs((self.train.target_speed - current_speed).abs());
            self.train.head_velocity = self.train.head_velocity.normalize_or_zero() * speed;

            // Movement
            head.collider.position += self.train.head_velocity * delta_time;
            head.collider.rotation = self.train.head_velocity.arg();
        }
    }
}
