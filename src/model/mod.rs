mod collider;
mod config;
mod logic;
mod particles;
mod train;
mod types;

pub use self::{collider::*, config::*, particles::*, train::*, types::*};

use crate::prelude::*;

pub struct Model {
    pub context: Context,
    pub config: Config,

    pub camera: Camera2d,
    pub map_bounds: Aabb2<Coord>,

    pub real_time: FloatTime,
    pub round_time: FloatTime,

    pub money: Money,
    pub resources: AssocList<ResourceKind, ResourceCount>,

    pub phase: Phase,
    pub deck: Deck,
    pub train: Train,
    pub depo: Collider,
    pub shop: Vec<ShopItem>,

    pub items: StructOf<Arena<Item>>,
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
            resources: AssocList::new(),

            phase: Phase::Setup,
            deck: Deck {
                wagons: config.train.starter_wagons.clone(),
            },
            train: Train {
                in_depo: false,
                target_speed: r32(0.0),
                train_speed: r32(0.0),
                wagons: vec![].into(),
            },
            depo: Collider::aabb(Aabb2::ZERO),
            shop: Vec::new(),

            items: default(),
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
