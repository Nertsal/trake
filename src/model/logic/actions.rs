use super::*;

impl Model {
    pub fn launch_train(&mut self) {
        let speed = self.config.train.rail_speed;
        self.train.target_speed = speed;
        self.train.train_speed = speed;
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
