use super::*;

impl Model {
    pub fn init(&mut self) {
        // Camera
        self.camera.center = self
            .grid
            .grid_to_world(self.config.map_size / 2 + vec2(1, 1))
            .as_f32();

        // Train
        self.train = Train {
            target_speed: r32(0.0),
            train_speed: r32(0.0),
            blocks: vec![TrainBlock::new_locomotive(
                &self.config.train,
                vec2(2.0, 3.0).as_r32(),
            )]
            .into(),
        };
        self.add_wagon(TrainBlockKind::Wagon);
        self.add_wagon(TrainBlockKind::Wagon);

        // Walls
        self.grid_items = default();
        let mut set_wall_at = |position: vec2<ICoord>| {
            self.grid_items.insert(GridItem {
                position,
                rail: None,
                resource: None,
                wall: Some(Wall {
                    collider: Collider::aabb(
                        Aabb2::point(self.grid.grid_to_world(position))
                            .extend_symmetric(self.grid.cell_size * r32(0.9 / 2.0)),
                    ),
                }),
            });
        };
        for x in 0..=self.config.map_size.x + 1 {
            set_wall_at(vec2(x, 0));
            set_wall_at(vec2(x, self.config.map_size.y + 1));
        }
        for y in 1..=self.config.map_size.y {
            set_wall_at(vec2(0, y));
            set_wall_at(vec2(self.config.map_size.x + 1, y));
        }
    }
}
