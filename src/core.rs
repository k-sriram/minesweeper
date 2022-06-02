use rand::Rng;
use std::{
    cmp::{max, min},
    fmt::Display,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    New,
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cell {
    Hidden,
    Mine,
    Clear(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenInfo {
    pub state: GameState,
    pub cell: Cell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenErr {
    OutOfBounds,
    AlreadyOpen,
    GameOver,
}

impl std::error::Error for OpenErr {}

impl Display for OpenErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpenErr::OutOfBounds => write!(f, "Out of bounds"),
            OpenErr::AlreadyOpen => write!(f, "Already open"),
            OpenErr::GameOver => write!(f, "Game over"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    state: GameState,
    width: u32,
    height: u32,
    num_mines: u32,
    randomizer: fn(u32, u32, u32, u32, u32) -> Vec<Vec<bool>>,
    clear_remaining: u32,
    mines: Vec<Vec<bool>>,
    opened: Vec<Vec<bool>>,
    neighbours: Vec<Vec<u8>>,
}

impl Game {
    pub fn new(width: u32, height: u32, num_mines: u32) -> Self {
        Self::new_with(height, width, num_mines, default_randomizer)
    }

    pub fn new_with(
        width: u32,
        height: u32,
        num_mines: u32,
        randomizer: fn(u32, u32, u32, u32, u32) -> Vec<Vec<bool>>,
    ) -> Self {
        if width < 1 || height < 1 {
            panic!("Invalid size");
        }
        if num_mines < 1 || num_mines > (width * height - 1) {
            panic!("Invalid number of mines");
        }
        Self {
            state: GameState::New,
            width,
            height,
            num_mines,
            randomizer,
            clear_remaining: 0,
            mines: Vec::new(),
            opened: Vec::new(),
            neighbours: Vec::new(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn num_mines(&self) -> u32 {
        self.num_mines
    }

    pub fn clear(&mut self) {
        self.state = GameState::New;
        self.mines = Vec::new();
        self.opened = Vec::new();
        self.neighbours = Vec::new();
    }

    pub fn reset(&mut self, width: u32, height: u32, num_mines: u32) {
        self.height = height;
        self.width = width;
        self.num_mines = num_mines;
        self.clear();
    }

    pub fn reset_randomizer(&mut self, randomizer: fn(u32, u32, u32, u32, u32) -> Vec<Vec<bool>>) {
        self.randomizer = randomizer;
    }

    pub fn get_state(&self) -> GameState {
        self.state
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Cell {
        match self.state {
            GameState::New => Cell::Hidden,
            GameState::Playing if !self.opened[y as usize][x as usize] => Cell::Hidden,
            _ => {
                if self.mines[y as usize][x as usize] {
                    Cell::Mine
                } else {
                    Cell::Clear(self.neighbours[y as usize][x as usize])
                }
            }
        }
    }

    pub fn get_board(&self) -> Vec<Vec<Cell>> {
        (0..self.height)
            .map(|y| (0..self.width).map(|x| self.get_cell(x, y)).collect())
            .collect()
    }

    pub fn open(&mut self, x: u32, y: u32) -> Result<OpenInfo, OpenErr> {
        if x >= self.width || y >= self.height {
            return Err(OpenErr::OutOfBounds);
        }
        match self.state {
            GameState::New => {
                self.state = GameState::Playing;
                self.generate_board(x, y);
                self.open(x, y)
            }
            GameState::Playing => {
                if self.opened[y as usize][x as usize] {
                    Err(OpenErr::AlreadyOpen)
                } else {
                    self.opened[y as usize][x as usize] = true;
                    if self.mines[y as usize][x as usize] {
                        self.state = GameState::Lost;
                        Ok(OpenInfo {
                            state: self.state,
                            cell: Cell::Mine,
                        })
                    } else {
                        self.clear_remaining -= 1;
                        if self.clear_remaining == 0 {
                            self.state = GameState::Won;
                        }
                        Ok(OpenInfo {
                            state: self.state,
                            cell: Cell::Clear(self.neighbours[y as usize][x as usize]),
                        })
                    }
                }
            }
            _ => Err(OpenErr::GameOver),
        }
    }

    // Utility functions

    fn generate_board(&mut self, firstx: u32, firsty: u32) {
        self.mines = (self.randomizer)(self.width, self.height, self.num_mines, firstx, firsty);
        self.opened = vec![vec![false; self.width as usize]; self.height as usize];
        self.calculate_neighbours();
        self.clear_remaining = self.width * self.height - self.num_mines;
    }

    fn calculate_neighbours(&mut self) {
        self.neighbours = vec![vec![0; self.width as usize]; self.height as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                if self.mines[y as usize][x as usize] {
                    for j in max(1, y as usize) - 1..min(self.height, y + 2) as usize {
                        for i in max(1, x as usize) - 1..min(self.width, x + 2) as usize {
                            self.neighbours[j][i] += 1;
                        }
                    }
                }
            }
        }
    }
}

fn default_randomizer(
    width: u32,
    height: u32,
    num_mines: u32,
    firstx: u32,
    firsty: u32,
) -> Vec<Vec<bool>> {
    let mut mines = vec![vec![false; width as usize]; height as usize];
    let mut mines_left = num_mines;
    let area = width * height;
    let spaces = area - num_mines - 1;
    mines[firsty as usize][firstx as usize] = true;

    let mut rng = rand::thread_rng();
    while mines_left > 0 {
        let r = rng.gen_range(0..spaces + mines_left);
        let mut i = 0;
        for j in 0..area {
            let (x, y) = (j % width, j / width);
            if mines[y as usize][x as usize] {
                continue;
            }
            if i == r {
                mines[y as usize][x as usize] = true;
                mines_left -= 1;
                break;
            }
            i += 1;
        }
    }
    mines[firsty as usize][firstx as usize] = false;
    mines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_randomizer(
        width: u32,
        height: u32,
        num_mines: u32,
        _firstx: u32,
        _firsty: u32,
    ) -> Vec<Vec<bool>> {
        let mut mines = vec![vec![false; width as usize]; height as usize];
        for i in 0..num_mines {
            let x = i % width;
            let y = i / width;
            mines[y as usize][x as usize] = true;
        }
        mines
    }

    #[test]
    fn run() {
        let mut game = Game::new_with(5, 4, 10, dummy_randomizer);
        assert_eq!(game.height(), 4);
        assert_eq!(game.width(), 5);
        assert_eq!(game.num_mines(), 10);
        assert_eq!(game.get_state(), GameState::New);
        assert_eq!(
            game.open(0, 2),
            Ok(OpenInfo {
                state: GameState::Playing,
                cell: Cell::Clear(2)
            })
        );
        assert_eq!(game.get_state(), GameState::Playing);
        assert_eq!(game.get_cell(0, 2), Cell::Clear(2));
        assert_eq!(game.get_cell(0, 3), Cell::Hidden);
        assert_eq!(game.open(0, 2), Err(OpenErr::AlreadyOpen));
        assert_eq!(game.open(4, 4), Err(OpenErr::OutOfBounds));
        assert_eq!(game.open(5, 3), Err(OpenErr::OutOfBounds));
        assert_eq!(
            game.open(1, 1),
            Ok(OpenInfo {
                state: GameState::Lost,
                cell: Cell::Mine
            })
        );
        assert_eq!(game.get_state(), GameState::Lost);
        assert_eq!(game.open(1, 2), Err(OpenErr::GameOver));
        assert_eq!(game.get_cell(0, 0), Cell::Mine);
        assert_eq!(game.get_cell(3, 3), Cell::Clear(0));

        // Game 2
        game.reset(3, 1, 1);
        assert_eq!(game.get_state(), GameState::New);
        assert_eq!(
            game.open(2, 0),
            Ok(OpenInfo {
                state: GameState::Playing,
                cell: Cell::Clear(0)
            })
        );
        assert_eq!(game.open(0, 1), Err(OpenErr::OutOfBounds));
        assert_eq!(
            game.open(1, 0),
            Ok(OpenInfo {
                state: GameState::Won,
                cell: Cell::Clear(1)
            })
        );

        assert_eq!(game.get_state(), GameState::Won);
        assert_eq!(game.open(1, 0), Err(OpenErr::GameOver));

        assert_eq!(
            game.get_board(),
            vec![vec![Cell::Mine, Cell::Clear(1), Cell::Clear(0)]]
        );

        game.clear();
        assert_eq!(game.get_state(), GameState::New);
    }

    #[test]
    fn game_new() {
        Game::new(5, 4, 10);
    }

    #[test]
    #[should_panic]
    fn invalid_size() {
        Game::new(0, 5, 0);
    }

    #[test]
    #[should_panic]
    fn invalid_mines_1() {
        Game::new(3, 3, 9);
    }

    #[test]
    #[should_panic]
    fn invalid_mines_2() {
        Game::new(3, 3, 0);
    }

    #[test]
    fn default_randomizer_basic() {
        let mines = default_randomizer(5, 4, 10, 0, 0);
        assert_eq!(
            mines
                .into_iter()
                .map(|v| v.into_iter().filter(|&x| x).count())
                .sum::<usize>(),
            10
        );
    }

    #[test]
    fn randomizer_does_not_get_first() {
        for i in 0..5 {
            let mines = default_randomizer(5, 5, 24, i, i);
            assert_eq!(mines[i as usize][i as usize], false);
        }
    }
}
