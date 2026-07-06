use dioxus::{
    fullstack::{CborEncoding, WebSocketOptions, Websocket},
    prelude::*,
};

mod game;
mod rooms;

use serde::{Deserialize, Serialize};

use thiserror::Error;
#[cfg(feature = "server")]
use tokio::sync::mpsc;

#[cfg(feature = "server")]
use crate::rooms::{Player, ROOMS, Room, unique_room_code};

#[cfg(feature = "server")]
use ulid::Ulid;

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

#[get("/api/create_room_ws?name")]
pub async fn create_room_ws(
    name: Option<String>,
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |mut socket| async move {
        let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();
        let player_x_id = Ulid::new().to_string();
        let player_x = Player::new(player_x_id.clone(), name, tx);

        let room_id = {
            let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
            let code = unique_room_code(&rooms);
            rooms.insert(code.clone(), Room::new(player_x));
            code
        };

        if socket
            .send(ServerEvent::RoomCreated {
                player_x_id,
                room_id: room_id.clone(),
            })
            .await
            .is_err()
        {
            return;
        }

        loop {
            tokio::select! {
                incoming = socket.recv() => {
                    match incoming {
                        Ok(ClientEvent::SetName(new_name)) => {
                            let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
                            if let Some(room) = rooms.get_mut(&room_id) {
                                room.player_x.name = Some(new_name);
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
                Some(event) = rx.recv() => {
                    if socket.send(event).await.is_err() {
                        break;
                    }
                }
            }
        }
    }))
}

#[cfg(feature = "server")]
enum GameConnect {
    NoRoom,
    NotMember,
    Connected {
        mark: &'static str,
        snapshot: ServerEvent,
    },
}

#[get("/api/game_ws?room_id&player_id")]
pub async fn game_ws(
    room_id: String,
    player_id: String,
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |mut socket| async move {
        let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

        let connect = {
            let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
            match rooms.get_mut(&room_id) {
                None => GameConnect::NoRoom,
                Some(room) => match room.mark_of(&player_id) {
                    None => GameConnect::NotMember,
                    Some(mark) => {
                        room.attach(mark, tx);
                        GameConnect::Connected {
                            mark,
                            snapshot: room.snapshot(),
                        }
                    }
                },
            }
        };

        let mark = match connect {
            GameConnect::NoRoom => {
                let _ = socket.send(ServerEvent::RoomNotFound).await;
                return;
            }
            GameConnect::NotMember => {
                let _ = socket.send(ServerEvent::Unauthorized).await;
                return;
            }
            GameConnect::Connected { mark, snapshot } => {
                if socket
                    .send(ServerEvent::GameConnected {
                        your_mark: mark.to_string(),
                    })
                    .await
                    .is_err()
                {
                    return;
                }

                if socket.send(snapshot).await.is_err() {
                    return;
                }

                mark
            }
        };

        loop {
            tokio::select! {
                incoming = socket.recv() => {
                    match incoming {
                        Ok(ClientEvent::Move { r, c }) => {
                            let result = {
                                let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
                                match rooms.get_mut(&room_id) {
                                    Some(room) => match room.apply_move(mark, r, c) {
                                        Ok(()) => {
                                            // mover gets the new state too, via its own rx
                                            room.broadcast(room.snapshot());
                                            Ok(())
                                        }
                                        Err(e) => Err(e),
                                    },
                                    None => Err(GameError::InvalidInput("room closed".to_string())),
                                }
                            };

                            if result.is_err() && socket.send(ServerEvent::InvalidMove).await.is_err() {
                                break;
                            }
                        }
                        Ok(ClientEvent::SetName(new_name)) => {
                            let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
                            if let Some(room) = rooms.get_mut(&room_id) {
                                match mark {
                                    "X" => room.player_x.name = Some(new_name),
                                    _ => {
                                        if let Some(player_o) = room.player_o.as_mut() {
                                            player_o.name = Some(new_name);
                                        }
                                    }
                                }
                                room.broadcast(room.snapshot());
                            }
                        }
                        Err(_) => break,
                    }
                }
                event = rx.recv() => {
                    match event {
                        Some(event) => {
                            if socket.send(event).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    }))
}

#[get("/api/join_room_ws?room_id&name")]
pub async fn join_room_ws(
    room_id: String,
    name: Option<String>,
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |mut socket| async move {
        let (player_sender, mut player_reciever) = mpsc::unbounded_channel::<ServerEvent>();
        let player_o_id = Ulid::new().to_string();
        let player_o = Player::new(player_o_id.clone(), name, player_sender);

        let found = {
            let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
            match rooms.get_mut(&room_id) {
                Some(room) => {
                    room.player_o = Some(player_o);
                    let _ = room.player_x.tx.send(ServerEvent::PlayerJoined {
                        player_o_id: player_o_id.clone(),
                    });
                    true
                }
                None => false,
            }
        };

        if !found {
            let _ = socket.send(ServerEvent::RoomNotFound).await;
            return;
        }

        if socket
            .send(ServerEvent::RoomJoined {
                player_o_id,
                room_id: room_id.clone(),
            })
            .await
            .is_err()
        {
            return;
        }

        loop {
            tokio::select! {
                incoming = socket.recv() => {
                    match incoming {
                        Ok(ClientEvent::SetName(new_name)) => {
                            info!("new_name");
                            let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
                            if let Some(room) = rooms.get_mut(&room_id) {
                                if let Some(player_o) = room.player_o.as_mut() {
                                    player_o.name = Some(new_name);
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
                Some(event) = player_reciever.recv() => {
                    if socket.send(event).await.is_err() {
                        break;
                    }
                }
            }
        }
    }))
}
