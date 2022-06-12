mod game;
mod ui;

pub use game::{Action, Game};

pub trait UI {
    fn get_action(&mut self, game: &Game) -> Action;
}

pub struct Engine {
    game: game::Game,
    ui: Box<dyn UI>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            game: game::Game::default(),
            ui: Box::new(ui::CLUI::new()),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.ui.get_action(&self.game) {
                Action::Quit => break,
                action => self.game.action(action).unwrap(),
            };
        }
    }
}
