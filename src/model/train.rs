use super::*;

#[derive(Debug, Clone)]
pub struct Train {
    pub in_depo: bool,
    pub target_speed: Coord,
    pub train_speed: Coord,
    pub fuel: Fuel,
    pub wagons: VecDeque<Wagon>,
}

#[derive(Debug, Clone)]
pub struct Wagon {
    pub status: WagonStatus,
    pub collider: Collider,
}

impl Wagon {
    pub fn new(position: vec2<Coord>, stats: WagonStats) -> Self {
        Self {
            collider: Collider::aabb(
                Aabb2::point(position).extend_symmetric(stats.size / r32(2.0)),
            ),
            status: WagonStatus {
                size: stats.size,
                health: Bounded::new_max(stats.max_health),
                fuel_capacity: stats.fuel_capacity,
                collect: stats.collect.map(|stats| WagonCollectStatus {
                    stats,
                    collecting: None,
                }),
            },
        }
    }
}

impl Train {
    pub fn fuel_capacity(&self) -> Fuel {
        r32(self
            .wagons
            .iter()
            .map(|wagon| wagon.status.fuel_capacity.as_f32())
            .sum())
    }
}

#[derive(Debug, Clone)]
pub struct WagonStatus {
    pub size: vec2<Coord>,
    pub health: Bounded<Hp>,
    pub fuel_capacity: Fuel,
    pub collect: Option<WagonCollectStatus>,
}

#[derive(Debug, Clone)]
pub struct WagonCollectStatus {
    pub stats: WagonCollectStats,
    pub collecting: Option<WagonCollecting>,
}

#[derive(Debug, Clone)]
pub struct WagonCollecting {
    /// Id of the resource being collected.
    pub resource: ArenaId,
    /// A number between 0 and 1.
    /// When it reaches 1, the resource gets collected.
    pub completion: Bounded<R32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WagonStats {
    pub size: vec2<Coord>,
    pub max_health: Hp,
    pub fuel_capacity: Fuel,
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
