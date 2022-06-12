use crate::game::{
    Cell, Game,
    GameState::{self, *},
};
use std::io::{self, BufRead};

pub enum Input {
    Quit,
    Reset,
    Flag(usize, usize),
    Open(usize, usize),
}

pub struct UI {}

impl UI {
    pub fn new() -> Self {
        UI {}
    }

    pub fn get_input(&mut self, game: &Game) -> Input {
        self.print_board(game);
        loop {
            let intext = io::stdin().lock().lines().next().unwrap().unwrap();
            if intext.is_empty() {
                println!("Please enter a valid command.");
                continue;
            }
            let c = intext.chars().next().unwrap();
            match c {
                'q' => return Input::Quit,
                'r' => return Input::Reset,
                'f' | 'o' => {
                    let (x, y) = self.parse_coordinates(intext);
                    if x == None || y == None {
                        println!("Please enter a valid command.");
                        continue;
                    }
                    let (x, y) = (x.unwrap(), y.unwrap());
                    if c == 'f' {
                        return Input::Flag(x, y);
                    } else {
                        return Input::Open(x, y);
                    }
                }
                _ => println!("Please enter a valid command."),
            }
        }
    }

    fn print_board(&self, game: &Game) {
        for row in game.board() {
            for cell in row {
                match cell {
                    Cell::Hidden => print!("."),
                    Cell::Flag => print!("F"),
                    Cell::Mine => print!("X"),
                    Cell::FalseFlag => print!("?"),
                    Cell::TrippedMine => print!("!"),
                    Cell::Open(x) => print!("{}", x),
                }
            }
            println!();
        }
        println!("{}", game.time_as_secs());
    }

    fn parse_coordinates(&self, intext: String) -> (Option<usize>, Option<usize>) {
        let mut splits = intext.split_whitespace();
        splits.next();
        let x = splits.next();
        let y = splits.next();
        (
            x.and_then(|x| x.parse::<usize>().ok()),
            y.and_then(|y| y.parse::<usize>().ok()),
        )
    }
}
