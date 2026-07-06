use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Easy,
    Hard,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won { mark: String },
    Draw,
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Invalid move")]
    InvalidMove(usize, usize),
    #[error("{0}")]
    InvalidInput(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientEvent {
    SetName(String),
    Move { r: usize, c: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    RoomCreated {
        player_x_id: String,
        room_id: String,
    },
    RoomJoined {
        player_o_id: String,
        room_id: String,
    },
    PlayerJoined {
        player_o_id: String,
    },
    RoomNotFound,
    Unauthorized,
    GameConnected {
        your_mark: String,
    },
    GameState {
        board: Vec<Vec<String>>,
        is_x_turn: bool,
        status: GameStatus,
        player_x_name: Option<String>,
        player_o_name: Option<String>,
    },
    InvalidMove,
}
