use super::*;

#[derive(Debug, Clone)]
pub enum GameAction {
    LaunchTrain,
    BuyShop(usize),
}

impl GameState {
    pub fn execute(&mut self, action: GameAction) {
        log::trace!("Executing {:?}", action);
        match action {
            GameAction::LaunchTrain => self.model.launch_train(),
            GameAction::BuyShop(i) => self.model.buy_shop(i),
        }
    }
}
