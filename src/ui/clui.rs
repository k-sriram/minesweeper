use crate::{game::{
    Action::{self, *},
    Cell, Game,
    GameState::{self, *},
}, UI};
use std::io::{self, BufRead};

pub struct CLUI {}

impl CLUI {
    pub fn new() -> Self {
        CLUI {}
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

impl UI for CLUI {
    fn get_action(&mut self, game: &Game) -> Action {
        self.print_board(game);
        loop {
            let intext = io::stdin().lock().lines().next().unwrap().unwrap();
            if intext.is_empty() {
                println!("Please enter a valid command.");
                continue;
            }
            let c = intext.chars().next().unwrap();
            match c {
                'q' => return Action::Quit,
                'r' => return Action::Reset,
                'f' | 'o' => {
                    let (x, y) = self.parse_coordinates(intext);
                    if x == None || y == None {
                        println!("Please enter a valid command.");
                        continue;
                    }
                    let (x, y) = (x.unwrap(), y.unwrap());
                    if c == 'f' {
                        return Action::Flag(x, y);
                    } else {
                        return Action::Open(x, y);
                    }
                }
                _ => println!("Please enter a valid command."),
            }
        }
    }
}