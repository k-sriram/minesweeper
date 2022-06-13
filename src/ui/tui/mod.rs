use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{Action, Game, UI, game::Difficulty};

const TICK: Duration = Duration::from_millis(100);

pub struct TUI {
    state: State,
    status: String,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Debug)]
enum State {
    Game(GameCursor),
    Settings(SettingsCursor),
}

impl Default for State {
    fn default() -> Self {
        State::Game(GameCursor::default())
    }
}

#[derive(Debug, Default)]
struct GameCursor {
    x: usize,
    y: usize,
}

#[derive(Debug)]
enum SettingsCursor {
    Easy,
    Medium,
    Hard,
    Custom,
    CustomEdit(CEState),
}

#[derive(Debug)]
struct CEState {
    old_difficulty: Difficulty,
    stage: EditDifficultyStage,
    width: usize,
    height: usize,
    mines: usize,
}

#[derive(Debug)]
enum EditDifficultyStage {
    Width,
    Height,
    Mines,
}

impl TUI {
    pub fn new() -> Self {
        let terminal = initialize_terminal().expect("Terminal initialization failed");
        Self {
            state: State::default(),
            status: String::default(),
            terminal,
        }
    }

    fn draw(&self, game: &Game) {
        todo!()
    }

    fn handle_event(&mut self, event: Event, game: &Game) -> Option<Action> {
        todo!()
    }
}

impl Drop for TUI {
    fn drop(&mut self) {
        deinitialize_terminal(&mut self.terminal).expect("Error deinitializing terminal");
    }
}

impl UI for TUI {
    fn get_action(&mut self, game: &Game) -> Action {
        loop {
            self.draw(game);
            if event::poll(TICK).unwrap() {
                let event = event::read().unwrap();
                match self.handle_event(event, game) {
                    Some(action) => return action,
                    None => continue,
                }
            }
        }
    }
}

fn initialize_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn deinitialize_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), io::Error> {
    flush_events()?;
    terminal.show_cursor()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn flush_events() -> Result<(), io::Error> {
    while event::poll(Duration::ZERO)? {
        event::read()?;
    }
    Ok(())
}
