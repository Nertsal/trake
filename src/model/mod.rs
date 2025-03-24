mod collider;
mod logic;
mod particles;

pub use self::{collider::*, particles::*};

use crate::prelude::*;

pub type Coord = R32;
pub type ICoord = i64;
pub type FloatTime = R32;
pub type Money = i64;
pub type Score = i64;

#[derive(Debug, Clone)]
pub struct PlayerInput {
    pub turn: Coord,
}

#[allow(clippy::derivable_impls)]
impl Default for PlayerInput {
    fn default() -> Self {
        Self { turn: Coord::ZERO }
    }
}

#[derive(Debug, Clone)]
pub struct Train {
    pub in_depo: bool,
    pub target_speed: Coord,
    pub train_speed: Coord,
    pub blocks: VecDeque<TrainBlock>,
}

#[derive(Debug, Clone)]
pub struct TrainBlock {
    pub kind: TrainBlockKind,
    pub collider: Collider,
    pub snapped_to_rail: bool,
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
            snapped_to_rail: false,
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
    Coin,
    Diamond,
    PlusCent,
    GhostFuel,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub map_size: vec2<ICoord>,
    pub depo_size: vec2<Coord>,
    pub deck: Deck,
    pub train: TrainConfig,
    pub resources: HashMap<Resource, ResourceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    pub overtime_slowdown: Coord,
    pub turn_speed: Angle<Coord>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub enum Phase {
    Setup,
    Resolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub resources: Vec<Resource>,
    pub rails: Vec<RailKind>,
}

#[derive(Debug, Clone)]
pub enum Upgrade {
    Resource(Resource),
    Speed,
    Feather,
    Turning,
}

#[derive(Debug, Clone)]
pub struct ShopItem {
    pub upgrade: Upgrade,
    pub price: Money,
    pub can_purchase: bool,
}

pub struct Model {
    pub context: Context,
    pub config: Config,

    pub camera: Camera2d,
    pub grid: Grid,

    pub real_time: FloatTime,
    pub round_time: FloatTime,

    pub quotas_completed: usize,
    pub total_score: Score,
    pub current_quota: Score,
    pub quota_score: Score,
    pub quota_day: usize,
    pub round_score: Score,
    pub money: Money,

    pub phase: Phase,
    pub deck: Deck,
    pub train: Train,
    pub depo: Collider,
    pub shop: Vec<ShopItem>,

    pub grid_items: StructOf<Arena<GridItem>>,
    pub particles_queue: Vec<SpawnParticles>,
    pub particles: StructOf<Arena<Particle>>,
    pub floating_texts: StructOf<Arena<FloatingText>>,
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

            real_time: FloatTime::ZERO,
            round_time: FloatTime::ZERO,

            quotas_completed: 0,
            total_score: 0,
            current_quota: 0,
            quota_score: 0,
            quota_day: 0,
            round_score: 0,
            money: 0,

            phase: Phase::Setup,
            deck: config.deck.clone(),
            train: Train {
                in_depo: false,
                target_speed: r32(0.0),
                train_speed: r32(0.0),
                blocks: vec![].into(),
            },
            depo: Collider::aabb(Aabb2::ZERO),
            shop: Vec::new(),

            grid_items: default(),
            particles_queue: Vec::new(),
            particles: default(),
            floating_texts: default(),

            context,
            config,
        };
        model.init();
        model
    }
}
