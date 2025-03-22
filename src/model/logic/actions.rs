use super::*;

impl Model {
    pub fn place_rail(&mut self, position: vec2<ICoord>, orientation: RailOrientation) {
        if self.rails.iter().any(|rail| rail.position == position) {
            return;
        }

        self.rails.push(Rail {
            position,
            orientation,
        });
    }
}
