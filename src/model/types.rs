use super::*;

pub type Coord = R32;
pub type FloatTime = R32;
pub type Money = i64;
pub type MoneyFraction = R32;
pub type ResourceCount = i64;
pub type Hp = R32;
pub type Fuel = R32;

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

#[derive(Debug, Clone)]
pub enum EntityAi {
    Shooter(ShooterAi),
}

#[derive(Debug, Clone)]
pub struct ShooterAi {
    /// How far away can the entity target enemies.
    pub range: Coord,
    /// How many times per second can this entity shoot.
    pub shooting_speed: R32,
    /// Value in range `0.0..=1.0`.
    /// When it reaches `0.0` the entity can shoot again.
    pub cooldown: Bounded<R32>,
    pub bullet_speed: Coord,
    pub bullet_damage: Hp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    Player,
    Enemy,
}

#[derive(SplitFields, Debug, Clone)]
pub struct Entity {
    pub collider: Collider,
    pub velocity: vec2<Coord>,
    pub health: Option<Bounded<Hp>>,
    pub team: Option<Team>,
    pub damage_on_collision: Option<Hp>,
    pub ai: Option<EntityAi>,
}

#[derive(SplitFields, Debug, Clone)]
pub struct Item {
    pub position: vec2<Coord>,
    pub resource: Option<ResourceNode>,
    pub wall: Option<Wall>,
}

#[derive(Debug, Clone)]
pub enum Phase {
    Starting,
    Action,
    Finished,
    Leaving { tunnel: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub wagons: Vec<WagonStats>,
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

#[derive(Debug, Clone)]
pub struct Tunnel {
    pub collider: Collider,
    pub name: Name,
    pub effects: Vec<TunnelEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelEffect {
    /// Strong wind.
    Wind,
    /// Slippery snow piles on the map.
    Snow,
    /// Game time is sped up.
    TimeWarp,
    /// Big rocks on the map.
    Rocks,
    /// Enemy ambush.
    Ambush,
    /// Extra resource nodes of a specific kind.
    ExtraResource(ResourceKind),
    /// Repair station in the middle of the map.
    RepairStation,
    /// Random map.
    Random,
    /// Shop map.
    Station,
    /// Boss.
    Boss,
}
