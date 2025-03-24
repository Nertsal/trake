use super::*;

#[derive(Debug, Clone)]
pub enum GameAction {
    LaunchTrain,
}

impl GameState {
    pub fn execute(&mut self, action: GameAction) {
        log::trace!("Executing {:?}", action);
        match action {
            GameAction::LaunchTrain => self.model.launch_train(),
        }
    }
}
