use minesweeper::{Engine, CLUI};

fn main() {
    let mut engine = Engine::new(Box::new(CLUI::new()));
    engine.run();
}
