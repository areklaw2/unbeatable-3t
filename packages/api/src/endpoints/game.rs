use dioxus::{
    fullstack::{CborEncoding, WebSocketOptions, Websocket},
    prelude::*,
};

use crate::{ClientEvent, ServerEvent};

#[cfg(feature = "server")]
use crate::{GameError, endpoints::GameSocket, registry::with_room};

#[cfg(feature = "server")]
use tokio::sync::mpsc::{self, UnboundedSender};

#[get("/api/game_ws?room_id&player_id")]
pub async fn game_ws(
    room_id: String,
    player_id: String,
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |socket| handle(socket, room_id, player_id)))
}

#[cfg(feature = "server")]
enum Connect {
    NoRoom,
    NotMember,
    Connected {
        mark: &'static str,
        snapshot: ServerEvent,
    },
}

#[cfg(feature = "server")]
fn connect(room_id: &str, player_id: &str, tx: UnboundedSender<ServerEvent>) -> Connect {
    with_room(room_id, |room| match room.mark_of(player_id) {
        None => Connect::NotMember,
        Some(mark) => {
            room.attach(mark, tx);
            Connect::Connected {
                mark,
                snapshot: room.snapshot(),
            }
        }
    })
    .unwrap_or(Connect::NoRoom)
}

// apply the move and push the new state to both players
#[cfg(feature = "server")]
fn try_move(room_id: &str, mark: &str, r: usize, c: usize) -> Result<(), GameError> {
    with_room(room_id, |room| {
        room.apply_move(mark, r, c)?;
        room.broadcast(room.snapshot());
        Ok(())
    })
    .unwrap_or(Err(GameError::InvalidInput("room closed".to_string())))
}

#[cfg(feature = "server")]
fn rename(room_id: &str, mark: &str, name: String) {
    with_room(room_id, |room| {
        room.set_name(mark, name);
        room.broadcast(room.snapshot());
    });
}

#[cfg(feature = "server")]
async fn handle(mut socket: GameSocket, room_id: String, player_id: String) {
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

    let mark = match connect(&room_id, &player_id, tx) {
        Connect::NoRoom => {
            let _ = socket.send(ServerEvent::RoomNotFound).await;
            return;
        }
        Connect::NotMember => {
            let _ = socket.send(ServerEvent::Unauthorized).await;
            return;
        }
        Connect::Connected { mark, snapshot } => {
            let connected = ServerEvent::GameConnected {
                your_mark: mark.to_string(),
            };
            if socket.send(connected).await.is_err() {
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
            incoming = socket.recv() => match incoming {
                Ok(ClientEvent::Move { r, c }) => {
                    // mover gets the new state too, via its own rx
                    if try_move(&room_id, mark, r, c).is_err()
                        && socket.send(ServerEvent::InvalidMove).await.is_err()
                    {
                        break;
                    }
                }
                Ok(ClientEvent::SetName(name)) => rename(&room_id, mark, name),
                Err(_) => break,
            },
            event = rx.recv() => match event {
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
