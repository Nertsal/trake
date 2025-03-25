mod collider;
mod logic;
mod particles;

pub use self::{collider::*, particles::*};

use crate::prelude::*;

pub type Coord = R32;
pub type FloatTime = R32;
pub type Money = i64;

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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub map_size: vec2<Coord>,
    pub depo_size: vec2<Coord>,
    pub train: TrainConfig,
    pub resources: HashMap<Resource, ResourceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    pub overtime_slowdown: Coord,
    pub turn_speed: Angle<Coord>,
    pub speed: Coord,
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
pub struct Wall {
    pub collider: Collider,
}

#[derive(SplitFields, Debug, Clone)]
pub struct Item {
    pub position: vec2<Coord>,
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
    pub wagons: Vec<TrainBlockKind>,
}

#[derive(Debug, Clone)]
pub enum Upgrade {
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
    pub map_bounds: Aabb2<Coord>,

    pub real_time: FloatTime,
    pub round_time: FloatTime,

    pub money: Money,

    pub phase: Phase,
    pub deck: Deck,
    pub train: Train,
    pub depo: Collider,
    pub shop: Vec<ShopItem>,

    pub grid_items: StructOf<Arena<Item>>,
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
                fov: Camera2dFov::Vertical(16.0),
            },
            map_bounds: Aabb2::ZERO.extend_positive(config.map_size),

            real_time: FloatTime::ZERO,
            round_time: FloatTime::ZERO,

            money: 0,

            phase: Phase::Setup,
            deck: Deck {
                wagons: vec![TrainBlockKind::Locomotive],
            },
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
