use super::*;

#[derive(Debug, Clone)]
pub struct Train {
    pub in_depo: bool,
    pub target_speed: Coord,
    pub train_speed: Coord,
    pub wagons: VecDeque<Wagon>,
}

#[derive(Debug, Clone)]
pub struct Wagon {
    pub stats: WagonStats,
    pub collider: Collider,
}

impl Wagon {
    pub fn new(config: &TrainConfig, position: vec2<Coord>, stats: WagonStats) -> Self {
        Self {
            collider: Collider::aabb(
                Aabb2::point(position).extend_symmetric(stats.size / r32(2.0)),
            ),
            stats,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WagonStats {
    pub size: vec2<Coord>,
    pub max_health: Hp,
    pub collect: Option<WagonCollectStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WagonCollectStats {
    /// The resource type that can be collected.
    pub resource: ResourceKind,
    /// How far away from the wagon can resources be collected.
    pub range: Coord,
    /// How many times per second a resource is collected (by default).
    pub speed: R32,
    /// How many resources can this wagon hold.
    pub capacity: ResourceCount,
}
