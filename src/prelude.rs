pub use crate::{assets::*, context::*, ui::layout::AreaOps};

pub use anyhow::Result;
pub use geng::prelude::*;
pub use geng_utils::{bounded::Bounded, conversions::*};
pub use itertools::*;
pub use stecs::{
    prelude::*,
    storage::arena::{Arena, ArenaId},
};
pub use time::Duration;

pub use std::collections::VecDeque;

pub type Color = Rgba<f32>;
pub type Name = Arc<str>;
