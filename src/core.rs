use std::fmt::Display;

pub trait Randomizer {
    fn generate_board(&self, width: u32, height: u32, num_mines: u32) -> Vec<Vec<bool>>;
}

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
pub struct Game<R: Randomizer> {
    state: GameState,
    width: u32,
    height: u32,
    num_mines: u32,
    randomizer: R,
    mines: Vec<Vec<bool>>,
    opened: Vec<Vec<bool>>,
    neighbours: Vec<Vec<u8>>,
}

impl Game<DefaultRandomizer> {
    pub fn new(width: u32, height: u32, num_mines: u32) -> Self {
        todo!()
    }
}

impl<R: Randomizer> Game<R> {
    pub fn new_with(width: u32, height: u32, num_mines: u32, randomizer: R) -> Game<R> {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    pub fn reset(&mut self, width: u32, height: u32, num_mines: u32) {
        todo!()
    }

    pub fn get_state(&self) -> GameState {
        todo!()
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Cell {
        todo!()
    }

    pub fn get_board(&self) -> Vec<Vec<Cell>> {
        todo!()
    }

    pub fn open(&mut self, x: u32, y: u32) -> Result<OpenInfo, OpenErr> {
        todo!()
    }
}

pub struct DefaultRandomizer {}

impl Randomizer for DefaultRandomizer {
    fn generate_board(&self, width: u32, height: u32, num_mines: u32) -> Vec<Vec<bool>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRandomizer {}

    impl Randomizer for DummyRandomizer {
        fn generate_board(&self, width: u32, height: u32, num_mines: u32) -> Vec<Vec<bool>> {
            let mut mines = vec![vec![false; width as usize]; height as usize];
            for i in 0..num_mines {
                let x = i % width;
                let y = i / width;
                mines[y as usize][x as usize] = true;
            }
            mines
        }
    }

    #[test]
    fn run() {
        let mut game = Game::new_with(5, 4, 10, DummyRandomizer {});
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
        assert_eq!(game.open(0, 1), Err(OpenErr::GameOver));

        game.clear();
        assert_eq!(game.get_state(), GameState::New);
    }
}
