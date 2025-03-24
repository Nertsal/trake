use super::*;

impl Model {
    pub fn buy_shop(&mut self, i: usize) {
        if let Some(item) = self.shop.get_mut(i) {
            if item.can_purchase && self.money >= item.price {
                self.money -= item.price;
                item.can_purchase = false;
                match item.upgrade {
                    Upgrade::Resource(resource) => self.deck.resources.push(resource),
                    Upgrade::Speed => {
                        let mult = r32(1.2);
                        self.config.train.rail_speed *= mult;
                        self.config.train.offrail_speed *= mult;
                    }
                    Upgrade::Feather => {
                        self.config.train.overtime_slowdown *= r32(0.9);
                    }
                }
            }
        }
    }

    pub fn launch_train(&mut self) {
        let Phase::Setup = self.phase else { return };

        let speed = self.config.train.rail_speed;
        self.train.target_speed = speed;
        self.train.train_speed = speed;

        self.phase = Phase::Resolution;
    }

    pub fn place_rail(&mut self, position: vec2<ICoord>, orientation: RailOrientation) {
        if query!(self.grid_items, (&position)).any(|&pos| pos == position) {
            return;
        }

        self.grid_items.insert(GridItem {
            position,
            rail: Some(Rail { orientation }),
            resource: None,
            wall: None,
        });
    }
}
