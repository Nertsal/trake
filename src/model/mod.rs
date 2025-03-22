mod collider;
mod logic;

pub use self::collider::*;

use crate::prelude::*;

pub type Coord = R32;
pub type ICoord = i64;
pub type FloatTime = R32;

#[derive(Debug, Clone)]
pub struct Train {
    pub head_velocity: vec2<Coord>,
    pub blocks: VecDeque<TrainBlock>,
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

#[derive(Debug, Clone)]
pub struct Grid {
    pub cell_size: vec2<Coord>,
    pub origin: vec2<Coord>,
}

impl Grid {
    pub fn world_to_grid(&self, world: vec2<Coord>) -> vec2<ICoord> {
        let grid = (world - self.origin) / self.cell_size;
        grid.map(|x| x.round().as_f32() as ICoord)
    }

    pub fn grid_to_world(&self, grid: vec2<ICoord>) -> vec2<Coord> {
        grid.map(|x| r32(x as f32)) * self.cell_size + self.origin
    }
}

#[derive(Debug, Clone)]
pub struct Rail {
    pub position: vec2<ICoord>,
    pub orientation: RailOrientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connections {
    pub left: bool,
    pub bottom: bool,
    pub right: bool,
    pub top: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct RailOrientation {
    pub kind: RailKind,
    pub rotation: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailKind {
    Straight,
    Left,
}

impl From<RailOrientation> for Connections {
    fn from(value: RailOrientation) -> Self {
        let mut cons = match value.kind {
            RailKind::Straight => [true, false, true, false],
            RailKind::Left => [true, false, false, true],
        };
        let rotation = value.rotation % cons.len();
        cons.rotate_right(rotation);
        let [left, bottom, right, top] = cons;
        Self {
            left,
            bottom,
            right,
            top,
        }
    }
}

pub struct Model {
    pub config: Config,

    pub camera: Camera2d,
    pub train: Train,
    pub rails: Vec<Rail>,
    pub grid: Grid,
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
                head_velocity: vec2(1.0, 0.0).as_r32(),
                blocks: vec![TrainBlock {
                    collider: Collider::aabb(
                        Aabb2::point(vec2::ZERO)
                            .extend_symmetric(config.train.wagon_size / r32(2.0)),
                    ),
                    kind: TrainBlockKind::Locomotive,
                }]
                .into(),
            },
            rails: vec![],
            grid: Grid {
                cell_size: vec2::splat(1.0).as_r32(),
                origin: vec2::ZERO,
            },

            config,
        }
    }
}
