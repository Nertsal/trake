use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    pub map_size: vec2<Coord>,
    pub depo_size: vec2<Coord>,
    pub train: TrainConfig,
    pub resource: ResourcesConfig,
    pub resources: HashMap<ResourceKind, ResourceConfig>,
    pub tunnels: TunnelsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    pub starter_wagons: Vec<WagonStats>,

    pub wagon_spacing: Coord,

    pub turn_speed: Angle<Coord>,
    pub speed: Coord,
    pub acceleration: Coord,
    pub deceleration: Coord,
    pub fuel_consumption: Fuel,
    pub head_damage: Hp,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelsConfig {
    pub prefix: Vec<TunnelPreset>,
    pub suffix: Vec<TunnelPreset>,
    pub special: Vec<TunnelPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelPreset {
    pub name: Name,
    pub effects: Vec<TunnelEffect>,
}
