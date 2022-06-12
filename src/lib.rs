mod game;
mod ui;

pub struct Engine {
    game: game::Game,
    ui: ui::UI,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            game: game::Game::new(3, 3, 1),
            ui: ui::UI::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let input = self.ui.get_input(&self.game);
            match input {
                ui::Input::Quit => break,
                ui::Input::Reset => self.game.reset(),
                ui::Input::Flag(x, y) => {
                    self.game.flag(x, y);
                }
                ui::Input::Open(x, y) => {
                    self.game.open(x, y);
                }
            }
        }
    }
}
