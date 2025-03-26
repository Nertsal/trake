pub use crate::{assets::*, context::*};

pub use anyhow::Result;
pub use associated_list::AssocList;
pub use geng::prelude::*;
pub use geng_utils::{bounded::Bounded, conversions::*};
pub use itertools::*;
pub use stecs::{prelude::*, storage::arena::Arena};
pub use time::Duration;

pub use std::collections::VecDeque;

pub type Color = Rgba<f32>;
pub type Name = Arc<str>;
