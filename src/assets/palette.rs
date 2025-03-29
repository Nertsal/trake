use super::*;

use crate::{model::ResourceKind, prelude::Color};

#[derive(
    trake_derive::EnumField, geng::asset::Load, Debug, Clone, Serialize, Deserialize, PartialEq,
)]
#[load(serde = "toml")]
pub struct Palette {
    pub default_color: Color,
    pub background: Color,
    pub wagon_bottom: Color,
    pub wagon_top: Color,
    pub range_circle: Color,
    pub wall: Color,
    pub steam: Color,
    pub wind: Color,

    pub team_player: Color,
    pub team_enemy: Color,
    pub team_neutral: Color,

    #[enum_field(skip)]
    pub resources: HashMap<ResourceKind, Color>,

    pub text_positive: Color,
    pub text_negative: Color,

    pub dark: Color,
    pub light: Color,
    pub highlight: Color,
}
