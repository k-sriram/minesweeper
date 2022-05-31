pub trait Randomizer {
    fn generate_board(&self, width: u32, height: u32, mines: u32) -> Vec<Vec<bool>>;
}

pub enum GameState {
    New,
    Playing,
    Won,
    Lost,
}

pub enum CellState {
    Hidden,
    Mine,
    Clear(u8),
}

pub struct OpenInfo {
    pub state: GameState,
    pub opens: Vec<CellOpenInfo>,
}

pub struct CellOpenInfo {
    pub x: u32,
    pub y: u32,
    pub state: CellState,
}

pub enum OpenErr {
    OutOfBounds,
    AlreadyOpen,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game<R: Randomizer> {
    randomizer: R,
}

impl Game<DefaultRandomizer> {
    pub fn new(height: u32, width: u32, mines: u32) -> Self {
        todo!()
    }
}

impl<R: Randomizer> Game<R> {
    pub fn new_with(height: u32, width: u32, mines: u32, randomizer: R) -> Game<R> {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    pub fn reset(&mut self, height: u32, width: u32, mines: u32) {
        todo!()
    }

    pub fn get_state(&self) -> GameState {
        todo!()
    }

    pub fn get_cell(&self, x: u32, y: u32) -> CellState {
        todo!()
    }

    pub fn get_board(&self) -> Vec<Vec<CellState>> {
        todo!()
    }

    pub fn open(&mut self, x: u32, y: u32) -> Result<OpenInfo, OpenErr> {
        todo!()
    }
}

pub struct DefaultRandomizer {}

impl Randomizer for DefaultRandomizer {
    fn generate_board(&self, width: u32, height: u32, mines: u32) -> Vec<Vec<bool>> {
        todo!()
    }
}
