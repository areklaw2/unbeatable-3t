use std::{cmp, fmt};

use rand::{RngExt, rngs::ThreadRng};
use thiserror::Error;

pub enum Mode {
    Easy,
    Hard,
}

impl TryFrom<&str> for Mode {
    type Error = GameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "easy" => Ok(Mode::Easy),
            "hard" => Ok(Mode::Hard),
            _ => Err(GameError::InvalidInput(
                "Must choice Easy or Hard (any case)!".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerPick {
    X,
    O,
}

impl TryFrom<&str> for PlayerPick {
    type Error = GameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "X" => Ok(PlayerPick::X),
            "O" => Ok(PlayerPick::O),
            _ => Err(GameError::InvalidInput(
                "Must choice X or O (any case)!".to_string(),
            )),
        }
    }
}

impl From<PlayerPick> for &str {
    fn from(value: PlayerPick) -> Self {
        match value {
            PlayerPick::X => "X",
            PlayerPick::O => "O",
        }
    }
}

impl From<PlayerPick> for String {
    fn from(value: PlayerPick) -> Self {
        match value {
            PlayerPick::X => "X".to_owned(),
            PlayerPick::O => "O".to_owned(),
        }
    }
}

pub enum State {
    GameOver(String),
    InProgresss(String),
}

pub struct Game {
    board: Vec<Vec<String>>,
    rng: ThreadRng,
    mode: Mode,
    is_player_turn: bool,
    player_pick: PlayerPick,
    computer_pick: PlayerPick,
}

impl Game {
    pub fn new(mode: Mode, player_pick: PlayerPick) -> Self {
        let is_player_turn = match player_pick {
            PlayerPick::X => true,
            PlayerPick::O => false,
        };

        let computer_pick = match player_pick {
            PlayerPick::X => PlayerPick::O,
            PlayerPick::O => PlayerPick::X,
        };

        Self {
            board: vec![vec![String::from(" "); 3]; 3],
            rng: rand::rng(),
            mode,
            is_player_turn,
            player_pick,
            computer_pick,
        }
    }

    pub fn run(&mut self, player_move: Option<(usize, usize)>) -> State {
        if self.is_full() {
            return State::GameOver("Game is a draw".to_string());
        }

        if self.win(self.player_pick.into()) {
            return State::GameOver("Player is the Winner!".to_string());
        }

        if self.win(self.computer_pick.into()) {
            return State::GameOver("Computer is the Winner!".to_string());
        }

        match self.is_player_turn {
            true => match player_move {
                Some((r, c)) => {
                    if let Err(e) = self.player_turn(r, c) {
                        return State::InProgresss(format!("{}", e));
                    }
                }
                None => return State::InProgresss("No player move provided".to_string()),
            },
            false => {
                println!("Computer's move...");
                if let Err(e) = self.cpu_turn() {
                    return State::InProgresss(format!("{}", e));
                }
            }
        };

        self.is_player_turn = !self.is_player_turn;

        State::InProgresss("".to_string())
    }

    pub fn is_player_turn(&self) -> bool {
        self.is_player_turn
    }

    fn is_full(&self) -> bool {
        for r in 0..3 {
            for c in 0..3 {
                if self.board[r][c] == " " {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn win(&self, value: &str) -> bool {
        [
            // Columns
            [&self.board[0][0], &self.board[1][0], &self.board[2][0]],
            [&self.board[0][1], &self.board[1][1], &self.board[2][1]],
            [&self.board[0][2], &self.board[1][2], &self.board[2][2]],
            // Rows
            [&self.board[0][0], &self.board[0][1], &self.board[0][2]],
            [&self.board[1][0], &self.board[1][1], &self.board[1][2]],
            [&self.board[2][0], &self.board[2][1], &self.board[2][2]],
            // Diagonals
            [&self.board[0][0], &self.board[1][1], &self.board[2][2]],
            [&self.board[0][2], &self.board[1][1], &self.board[2][0]],
        ]
        .iter()
        .any(|combo| combo.iter().all(|&cell| cell == value))
    }

    fn player_turn(&mut self, r: usize, c: usize) -> Result<(), GameError> {
        if r >= 3 || c >= 3 {
            return Err(GameError::InvalidMove(r, c));
        }

        if self.board[r][c] != " " {
            return Err(GameError::InvalidMove(r, c));
        }

        self.board[r][c] = self.player_pick.into();
        Ok(())
    }

    fn cpu_turn(&mut self) -> Result<(), GameError> {
        match self.mode {
            Mode::Easy => self.random_cpu_turn(self.computer_pick.into()),
            Mode::Hard => self.minimax_cpu_turn(self.computer_pick.into()),
        }
    }

    fn random_cpu_turn(&mut self, value: &str) -> Result<(), GameError> {
        let mut empty_spots = Vec::new();
        for r in 0..3 {
            for c in 0..3 {
                if self.board[r][c] == " " {
                    empty_spots.push((r, c));
                }
            }
        }

        let index = self.rng.random_range(0..empty_spots.len());
        let (r, c) = empty_spots[index];
        self.board[r][c] = value.to_string();
        Ok(())
    }

    fn minimax_cpu_turn(&mut self, value: &str) -> Result<(), GameError> {
        let mut best_score = 1000;
        let mut best_move = (0, 0);
        for r in 0..3 {
            for c in 0..3 {
                if self.board[r][c] == " " {
                    self.board[r][c] = self.computer_pick.into();
                    let score = self.minimax(true, 0);
                    self.board[r][c] = " ".to_string();

                    if score < best_score {
                        best_move = (r, c);
                        best_score = score;
                    }
                }
            }
        }

        let (r, c) = best_move;
        self.board[r][c] = value.to_string();
        Ok(())
    }

    fn score(&self, depth: i32) -> i32 {
        if self.win(self.player_pick.into()) {
            return 10 - depth;
        } else if self.win(self.computer_pick.into()) {
            return depth - 10;
        } else {
            return 0;
        }
    }

    fn minimax(&mut self, is_maximizer: bool, depth: i32) -> i32 {
        let score = self.score(depth);
        if score > 0 || score < 0 || self.is_full() {
            return score;
        }

        match is_maximizer {
            true => {
                let mut best = -1000;
                for r in 0..3 {
                    for c in 0..3 {
                        if self.board[r][c] == " " {
                            self.board[r][c] = self.player_pick.into();
                            best = cmp::max(best, self.minimax(false, depth + 1));
                            self.board[r][c] = " ".to_string();
                        }
                    }
                }
                best
            }
            false => {
                let mut best = 1000;
                for r in 0..3 {
                    for c in 0..3 {
                        if self.board[r][c] == " " {
                            self.board[r][c] = self.computer_pick.into();
                            best = cmp::min(best, self.minimax(true, depth + 1));
                            self.board[r][c] = " ".to_string();
                        }
                    }
                }
                best
            }
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rows = Vec::new();
        for r in 0..3 {
            let row = self.board[r].join(" | ");
            rows.push(row);
        }
        let board = rows.join("\n---------\n");
        write!(f, "{}", board)
    }
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Invalid move")]
    InvalidMove(usize, usize),
    #[error("{0}")]
    InvalidInput(String),
}
