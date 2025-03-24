use super::*;

#[derive(Debug, Clone)]
pub enum GameAction {}

impl GameState {
    pub fn execute(&mut self, action: GameAction) {}
}
