use super::*;

use crate::prelude::Color;

#[derive(
    trake_derive::EnumField,
    geng::asset::Load,
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
)]
#[load(serde = "toml")]
pub struct Palette {
    pub background: Color,
    pub locomotive_bottom: Color,
    pub locomotive_top: Color,
    pub wall: Color,
    pub steam: Color,

    pub text_positive: Color,
    pub text_negative: Color,

    pub dark: Color,
    pub light: Color,
    pub highlight: Color,
}
