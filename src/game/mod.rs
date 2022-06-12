mod rules;
mod timer;

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
pub struct Game {
    rules: GameRules,
    timer: Timer,
    board: Vec<Vec<Cell>>,
    height: usize,
    width: usize,
    mines: usize,
    mines_remaining: usize,
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let rules = GameRules::new(width, height, mines);
        Game {
            rules,
            timer: Timer::new(),
            board: vec![vec![Cell::Hidden; width]; height],
            height,
            width,
            mines,
            mines_remaining: mines,
        }
    }

    pub fn reset(&mut self) {
        self.timer.reset();
        self.board = vec![vec![Cell::Hidden; self.width]; self.height];
        self.rules.clear();
        self.mines_remaining = self.mines;
    }

    // TODO: Return a Result
    pub fn open(&mut self, x: usize, y: usize) {
        if self.state() == New {
            self.timer.start();
        }

        if x >= self.width || y >= self.height {
            panic!("Out of bounds");
        }

        if self.state() == Lost || self.state() == Won {
            panic!("Game is over");
        }

        if self.board[y][x] == Cell::Flag {
            return;
        }

        match self.rules.open(x, y) {
            Err(rules::OpenErr::OutOfBounds) => panic!("Out of bounds"),
            Err(_) => {}
            Ok(info) => match info {
                rules::OpenInfo {
                    state: Playing,
                    cell,
                } => {
                    self.board[y][x] = cell.into();
                }
                rules::OpenInfo { state: _, cell } => {
                    if cell == rules::Cell::Mine {
                        self.board[y][x] = Cell::TrippedMine;
                    }
                    self.query_board();
                    self.timer.stop();
                }
            },
        }
    }

    pub fn flag(&mut self, x: usize, y: usize) {
        if self.board[y][x] == Cell::Hidden {
            self.board[y][x] = Cell::Flag;
            self.mines_remaining -= 1;
        } else if self.board[y][x] == Cell::Flag {
            self.board[y][x] = Cell::Hidden;
            self.mines_remaining += 1;
        }
    }

    pub fn state(&self) -> GameState {
        self.rules.get_state()
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
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn mines(&self) -> usize {
        self.mines
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
                        } else if *cell == Cell::Flag && self.rules.get_cell(x, y).is_clear() {
                            *cell = Cell::FalseFlag;
                        }
                    }
                }
            }
            _ => {
                panic!("query_board can only be called after game is over");
            }
        }
    }
}
