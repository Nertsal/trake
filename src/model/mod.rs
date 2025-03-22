mod collider;
mod logic;

pub use self::collider::*;

use crate::prelude::*;

pub type Coord = R32;

#[derive(Debug, Clone)]
pub struct Train {
    pub head_velocity: vec2<Coord>,
    pub blocks: Vec<TrainBlock>,
}

#[derive(Debug, Clone)]
pub struct TrainBlock {
    pub kind: TrainBlockKind,
    pub collider: Collider,
}

#[derive(Debug, Clone)]
pub enum TrainBlockKind {
    Locomotive,
    Wagon,
}

pub enum Resource {}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub train: TrainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    pub rail_speed: Coord,
    pub offrail_speed: Coord,
    pub acceleration: Coord,
    pub deceleration: Coord,
    pub wagon_size: vec2<Coord>,
}

pub struct Model {
    pub config: Config,

    pub camera: Camera2d,
    pub train: Train,
}

impl Model {
    pub fn new(config: Config) -> Self {
        Self {
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 16.0,
            },
            train: Train {
                head_velocity: vec2::ZERO,
                blocks: vec![TrainBlock {
                    collider: Collider::aabb(
                        Aabb2::point(vec2::ZERO)
                            .extend_symmetric(config.train.wagon_size / r32(2.0)),
                    ),
                    kind: TrainBlockKind::Locomotive,
                }],
            },

            config,
        }
    }
}
