mod collider;
mod logic;

pub use self::collider::*;

use crate::prelude::*;

pub type Coord = R32;
pub type ICoord = i64;
pub type FloatTime = R32;
pub type Money = i64;

#[derive(Debug, Clone)]
pub struct Train {
    pub target_speed: Coord,
    pub train_speed: Coord,
    pub blocks: VecDeque<TrainBlock>,
}

#[derive(Debug, Clone)]
pub struct TrainBlock {
    pub kind: TrainBlockKind,
    pub collider: Collider,
    pub entering_rail: bool,
    pub path: VecDeque<vec2<Coord>>,
}

impl TrainBlock {
    pub fn new_locomotive(config: &TrainConfig, position: vec2<Coord>) -> Self {
        Self::new(config, position, TrainBlockKind::Locomotive)
    }

    pub fn new_wagon(config: &TrainConfig, position: vec2<Coord>) -> Self {
        Self::new(config, position, TrainBlockKind::Wagon)
    }

    pub fn new(config: &TrainConfig, position: vec2<Coord>, kind: TrainBlockKind) -> Self {
        Self {
            collider: Collider::aabb(
                Aabb2::point(position).extend_symmetric(config.wagon_size / r32(2.0)),
            ),
            kind,
            entering_rail: false,
            path: VecDeque::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TrainBlockKind {
    Locomotive,
    Wagon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Coal,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub map_size: vec2<ICoord>,
    pub train: TrainConfig,
    pub resources: HashMap<Resource, ResourceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    pub rail_speed: Coord,
    pub offrail_speed: Coord,
    pub acceleration: Coord,
    pub deceleration: Coord,
    pub wagon_size: vec2<Coord>,
    pub wagon_spacing: Coord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub value: Money,
    pub rarity: R32,
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
        self.gridf_to_world(grid.map(|x| r32(x as f32)))
    }

    pub fn gridf_to_world(&self, grid: vec2<Coord>) -> vec2<Coord> {
        grid * self.cell_size + self.origin
    }
}

#[derive(Debug, Clone)]
pub struct Rail {
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

#[derive(Debug, Clone)]
pub struct Wall {
    pub collider: Collider,
}

#[derive(SplitFields, Debug, Clone)]
pub struct GridItem {
    pub position: vec2<ICoord>,
    pub rail: Option<Rail>,
    pub resource: Option<Resource>,
    pub wall: Option<Wall>,
}

pub struct Model {
    pub context: Context,
    pub config: Config,

    pub camera: Camera2d,
    pub grid: Grid,

    pub train: Train,
    pub grid_items: StructOf<Arena<GridItem>>,
}

impl Model {
    pub fn new(context: Context, config: Config) -> Self {
        let mut model = Self {
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 16.0,
            },
            grid: Grid {
                cell_size: vec2::splat(1.0).as_r32(),
                origin: vec2::ZERO,
            },

            train: Train {
                target_speed: r32(0.0),
                train_speed: r32(0.0),
                blocks: vec![].into(),
            },
            grid_items: default(),

            context,
            config,
        };
        model.init();
        model
    }
}
