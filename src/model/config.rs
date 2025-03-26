use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub map_size: vec2<Coord>,
    pub depo_size: vec2<Coord>,
    pub train: TrainConfig,
    pub resource: ResourcesConfig,
    pub resources: HashMap<ResourceKind, ResourceConfig>,
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
pub struct ResourcesConfig {
    pub spawn_time: FloatTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Conversion rate into gold.
    pub value: MoneyFraction,
    /// The total amount of resources left on this node.
    pub amount: ResourceCount,
    /// How many times per second this resource is collected (by default).
    pub speed: R32,
    /// How many resources to transfer per single collection.
    pub per_collection: ResourceCount,
}
