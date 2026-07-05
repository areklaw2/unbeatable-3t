use dioxus::{
    fullstack::{CborEncoding, WebSocketOptions, Websocket},
    prelude::*,
};

mod game;
mod rooms;

pub use game::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use tokio::sync::mpsc;

#[cfg(feature = "server")]
use crate::rooms::{Player, ROOMS, Room, unique_room_code};

#[cfg(feature = "server")]
use ulid::Ulid;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientEvent {
    SetName(String),
}

#[derive(Serialize, Deserialize, Debug)]
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

#[get("/api/join_room_ws?room_id&name")]
pub async fn join_room_ws(
    room_id: String,
    name: String,
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |mut socket| async move {
        let (player_sender, mut player_reciever) = mpsc::unbounded_channel::<ServerEvent>();
        let player_o_id = Ulid::new().to_string();
        let player_o = Player::new(player_o_id.clone(), Some(name), player_sender);

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
                            let mut rooms = ROOMS.lock().unwrap_or_else(|p| p.into_inner());
                            if let Some(room) = rooms.get_mut(&room_id) {
                                if let Some(player_o) = room.player_o.as_mut() {
                                    player_o.name = Some(new_name);
                                }
                            }
                        }
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
