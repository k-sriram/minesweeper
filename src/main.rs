use minesweeper::{Engine, Settings, TUI};

fn main() {
    let default_settings = Settings::default();
    let mut engine = Engine::new(Box::new(TUI::new(&default_settings)));
    engine.run();
}
