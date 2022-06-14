mod rules;
mod timer;

use std::cmp::{max, min};

pub use rules::GameState::{self, *};

use rules::GameRules;
use timer::Timer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Hidden,
    Flag,
    Mine,
    FalseFlag,
    TrippedMine,
    Open(u8),
}

impl From<rules::Cell> for Cell {
    fn from(cell: rules::Cell) -> Self {
        match cell {
            rules::Cell::Hidden => Cell::Hidden,
            rules::Cell::Mine => Cell::Mine,
            rules::Cell::Clear(x) => Cell::Open(x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    ChangeSettings(Settings),
    Reset,
    Open(usize, usize),
    Flag(usize, usize),
    Chord(usize, usize),
    OpenOrChord(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    rules: GameRules,
    timer: Timer,
    board: Vec<Vec<Cell>>,
    settings: Settings,
    mines_remaining: usize,
}

impl Game {
    pub fn new(settings: Settings) -> Self {
        let (width, height) = (settings.difficulty.width(), settings.difficulty.height());
        let mines = settings.difficulty.mines();
        let rules = GameRules::new(width, height, mines);
        Game {
            rules,
            timer: Timer::new(),
            board: vec![vec![Cell::Hidden; width]; height],
            settings,
            mines_remaining: mines,
        }
    }

    pub fn action(&mut self, action: Action) -> Result<(), &'static str> {
        match action {
            Action::ChangeSettings(settings) => self.change_settings(settings),
            Action::Reset => self.reset(),
            Action::Open(x, y) => self.open(x, y),
            Action::Flag(x, y) => self.flag(x, y),
            Action::Quit => Err("Quit should be processed by the engine."),
            Action::Chord(x, y) => self.chord(x, y),
            Action::OpenOrChord(x, y) => self.open_or_chord(x, y),
        }
    }

    fn change_settings(&mut self, settings: Settings) -> Result<(), &'static str> {
        let should_reset = self.settings.difficulty != settings.difficulty;
        self.settings = settings;
        if should_reset {
            let (width, height, num_mines) = (
                self.settings.difficulty.width(),
                self.settings.difficulty.height(),
                self.settings.difficulty.mines(),
            );
            self.rules.reset(width, height, num_mines);
            self.reset()?;
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<(), &'static str> {
        self.timer.reset();
        self.board = vec![vec![Cell::Hidden; self.width()]; self.height()];
        self.rules.clear();
        self.mines_remaining = self.mines();
        Ok(())
    }

    fn open_or_chord(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        if let Cell::Open(_) = self.board[y][x] {
            self.chord(x, y)
        } else {
            self.open(x, y)
        }
    }

    fn open(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        if !self.valid_coord(x, y) {
            return Err("invalid coordinate");
        }
        if self.state() == New {
            self.timer.start();
        }

        if self.state() == Lost || self.state() == Won {
            return Err("Game is over");
        }

        match self.board[y][x] {
            Cell::Flag => return Err("Cell if flagged"),
            Cell::Open(_) => return Err("Cell is already open"),
            Cell::Hidden => {
                match self.rules.open(x, y) {
                    Err(err) => {
                        panic!("Unreachable: fail conditions already checked got {:?}", err)
                    }
                    Ok(info) => match info {
                        rules::OpenInfo {
                            state: Playing,
                            cell,
                        } => {
                            self.board[y][x] = cell.into();
                            if cell == rules::Cell::Clear(0) {
                                self.open_neighbors(x, y);
                            }
                        }
                        rules::OpenInfo { state: _, cell } => {
                            if cell == rules::Cell::Mine {
                                self.board[y][x] = Cell::TrippedMine;
                            }
                            self.query_board();
                            self.timer.stop();
                        }
                    },
                };
                Ok(())
            }
            _ => panic!(
                "Unreachable: FalseFlag and TrippedMine conditions only possible after game over"
            ),
        }
    }

    fn open_neighbors(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        for (xn, yn) in self.neighbours(x, y) {
            if self.board[yn][xn] == Cell::Hidden {
                self.open(xn, yn)?;
            }
            if self.board[yn][xn] == Cell::TrippedMine {
                return Err("Auto-tripped on mine.");
            }
        }
        Ok(())
    }

    fn flag(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        if !self.valid_coord(x, y) {
            return Err("invalid coordinate");
        }
        match self.board[y][x] {
            Cell::Hidden => {
                self.board[y][x] = Cell::Flag;
                self.mines_remaining -= 1;
                Ok(())
            }
            Cell::Flag => {
                self.board[y][x] = Cell::Hidden;
                self.mines_remaining += 1;
                Ok(())
            }
            _ => Err("Cell not hidden"),
        }
    }

    fn chord(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        if !self.valid_coord(x, y) {
            return Err("invalid coordinate");
        }
        match self.board[y][x] {
            Cell::Flag => Err("Cell is flagged"),
            Cell::Hidden => Err("Cell is hidden"),
            Cell::Open(mines) => {
                if self.state() != Playing {
                    return Err("Game is over");
                }
                if mines == 0 {
                    return Err("0s get auto-chorded.");
                }
                let mut neighbouring_flags = 0;
                for (x, y) in self.neighbours(x, y) {
                    if self.board[y][x] == Cell::Flag {
                        neighbouring_flags += 1;
                    }
                }
                if neighbouring_flags == mines {
                    self.open_neighbors(x, y)
                } else {
                    Err("Can't chord this cell. Incorrect number of flags.")
                }
            }
            _ => Err("Game is over"),
        }
    }

    pub fn state(&self) -> GameState {
        self.rules.get_state()
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn time(&self) -> f64 {
        self.timer.time_f64()
    }

    pub fn time_as_secs(&self) -> u64 {
        self.timer.time_as_secs()
    }

    pub fn cell(&self, x: usize, y: usize) -> Cell {
        self.board[y][x]
    }

    pub fn board(&self) -> &Vec<Vec<Cell>> {
        &self.board
    }

    pub fn width(&self) -> usize {
        self.settings.difficulty.width()
    }

    pub fn height(&self) -> usize {
        self.settings.difficulty.height()
    }

    pub fn mines(&self) -> usize {
        self.settings.difficulty.mines()
    }

    pub fn mines_remaining(&self) -> usize {
        self.mines_remaining
    }

    fn query_board(&mut self) {
        match self.state() {
            Won | Lost => {
                for (y, row) in self.board.iter_mut().enumerate() {
                    for (x, cell) in row.iter_mut().enumerate() {
                        if *cell == Cell::Hidden {
                            *cell = self.rules.get_cell(x, y).into();
                        } else if *cell == Cell::Flag {
                            if self.rules.get_cell(x, y).is_clear() {
                                *cell = Cell::FalseFlag;
                            } else {
                                *cell = Cell::Mine;
                            }
                        }
                    }
                }
            }
            _ => {
                panic!("query_board can only be called after game is over");
            }
        }
    }

    fn valid_coord(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(Settings::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Custom(CustomBoard),
}

impl Difficulty {
    pub fn width(&self) -> usize {
        match self {
            Difficulty::Easy => 9,
            Difficulty::Medium => 16,
            Difficulty::Hard => 30,
            Difficulty::Custom(board) => board.width,
        }
    }

    pub fn height(&self) -> usize {
        match self {
            Difficulty::Easy => 9,
            Difficulty::Medium => 16,
            Difficulty::Hard => 16,
            Difficulty::Custom(board) => board.height,
        }
    }

    pub fn mines(&self) -> usize {
        match self {
            Difficulty::Easy => 10,
            Difficulty::Medium => 40,
            Difficulty::Hard => 99,
            Difficulty::Custom(board) => board.mines,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomBoard {
    width: usize,
    height: usize,
    mines: usize,
}

impl CustomBoard {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        if width < 1 || height < 1 {
            panic!("Width and height must be at least 1")
        } else if mines < 1 {
            panic!("Mines must be at least 1")
        } else if mines >= width * height {
            panic!("Mines must be less than the number of cells")
        } else {
            CustomBoard {
                width,
                height,
                mines,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub difficulty: Difficulty,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            difficulty: Difficulty::Easy,
        }
    }
}

struct NeighboursIter {
    neighbours: Vec<(usize, usize)>,
    index: usize,
}

impl Game {
    fn neighbours(&self, x: usize, y: usize) -> NeighboursIter {
        let mut neighbours = Vec::new();
        for yn in y.saturating_sub(1)..min(self.height(), y + 2) {
            for xn in x.saturating_sub(1)..min(self.width(), x + 2) {
                if xn == x && yn == y {
                    continue;
                }
                neighbours.push((xn, yn));
            }
        }
        NeighboursIter {
            neighbours,
            index: 0,
        }
    }
}

impl Iterator for NeighboursIter {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.neighbours.len() {
            None
        } else {
            let (x, y) = self.neighbours[self.index];
            self.index += 1;
            Some((x, y))
        }
    }
}
