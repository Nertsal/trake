use super::*;

impl Model {
    pub fn place_rail(&mut self, position: vec2<ICoord>, orientation: RailOrientation) {
        if query!(self.grid_items, (&position)).any(|&pos| pos == position) {
            return;
        }

        self.grid_items.insert(GridItem {
            position,
            rail: Some(Rail { orientation }),
            resource: None,
        });
    }
}
