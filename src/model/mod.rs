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
    pub simulation_time: FloatTime,
    pub round_simulation_time: FloatTime,
    pub game_time_scale: FloatTime,

    pub money: Money,

    pub phase: Phase,
    pub deck: Deck,
    pub shop: Vec<ShopItem>,

    pub train: Train,
    pub depo: Collider,
    pub tunnels: Vec<Tunnel>,
    pub items: StructOf<Arena<Item>>,
    pub entities: StructOf<Arena<Entity>>,
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
            simulation_time: FloatTime::ZERO,
            round_simulation_time: FloatTime::ZERO,
            game_time_scale: FloatTime::ONE,

            money: 0,

            phase: Phase::Starting,
            deck: Deck {
                wagons: config.train.starter_wagons.clone(),
            },
            shop: Vec::new(),

            train: Train {
                in_depo: false,
                target_speed: r32(0.0),
                train_speed: r32(0.0),
                head_damage: config.train.head_damage,
                wagons: vec![].into(),
                fuel: Fuel::ZERO,
            },
            depo: Collider::aabb(Aabb2::ZERO),
            tunnels: Vec::new(),
            items: default(),
            entities: default(),
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
