use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::engine::Board;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Easy,
    Hard,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    X,
    O,
}

impl Mark {
    pub fn other(self) -> Mark {
        match self {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mark::X => "X",
            Mark::O => "O",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won { mark: Mark },
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
    Move { cell: usize },
    Rematch,
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
        your_mark: Mark,
    },
    GameState {
        board: Board,
        is_x_turn: bool,
        status: GameStatus,
        player_x_name: Option<String>,
        player_o_name: Option<String>,
    },
    InvalidMove,
    OpponentPresence {
        connected: bool,
    },
    RematchRequested {
        mark: Mark,
    },
}
