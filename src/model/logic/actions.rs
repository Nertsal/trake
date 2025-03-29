use super::*;

impl Model {
    pub fn buy_shop(&mut self, i: usize) {
        if let Some(item) = self.shop.get_mut(i) {
            if item.can_purchase && self.money >= item.price {
                self.money -= item.price;
                item.can_purchase = false;
                match item.upgrade {
                    Upgrade::Speed => {
                        let mult = r32(1.2);
                        self.config.train.speed *= mult;
                    }
                    Upgrade::Feather => {}
                    Upgrade::Turning => {
                        let limit = Angle::from_radians(r32(4.0));
                        let s = &mut self.config.train.turn_speed;
                        *s = limit + (*s - limit) * r32(0.75);
                    }
                }
            }
        }
    }

    pub fn launch_train(&mut self) {
        let Phase::Starting = self.phase else { return };

        let speed = self.config.train.speed;
        self.train.target_speed = speed;
        self.train.train_speed = speed;

        self.phase = Phase::Action;
        self.context.play_sfx(&self.context.assets.sounds.choochoo);
    }

    pub fn choose_tunnel(&mut self, tunnel: usize) {
        let Phase::Finished = self.phase else { return };

        let Some(tunnel) = self.tunnels.get(tunnel) else {
            return;
        };

        // TODO: drive into the tunnel
        // self.phase = Phase::Leaving { tunnel };
        self.next_map(tunnel.clone());
    }
}
