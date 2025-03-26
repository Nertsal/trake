use super::*;

pub type Coord = R32;
pub type FloatTime = R32;
pub type Money = i64;
pub type MoneyFraction = R32;
pub type ResourceCount = i64;

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
pub enum ResourceKind {
    Wood,
    Coal,
    Food,
}

#[derive(Debug, Clone)]
pub enum ResourceNodeState {
    Spawning(Bounded<FloatTime>),
    Idle,
    Despawning(Bounded<FloatTime>),
}

#[derive(Debug, Clone)]
pub struct ResourceNode {
    pub kind: ResourceKind,
    pub data: ResourceConfig,
    pub state: ResourceNodeState,
}

#[derive(Debug, Clone)]
pub struct Wall {
    pub collider: Collider,
}

#[derive(SplitFields, Debug, Clone)]
pub struct Item {
    pub position: vec2<Coord>,
    pub resource: Option<ResourceNode>,
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
