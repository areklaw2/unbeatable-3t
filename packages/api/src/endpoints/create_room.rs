use dioxus::{
    fullstack::{CborEncoding, WebSocketOptions, Websocket},
    prelude::*,
};

use crate::{ClientEvent, ServerEvent};

#[cfg(feature = "server")]
use crate::{
    endpoints::GameSocket,
    registry::{insert_room, with_room},
    room::Player,
};

#[cfg(feature = "server")]
use tokio::sync::mpsc;

#[cfg(feature = "server")]
use ulid::Ulid;

#[get("/api/create_room_ws?name")]
pub async fn create_room_ws(
    name: Option<String>,
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |socket| handle(socket, name)))
}

#[cfg(feature = "server")]
async fn handle(mut socket: GameSocket, name: Option<String>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();
    let player_x_id = Ulid::new().to_string();
    let room_id = insert_room(Player::new(player_x_id.clone(), name, tx));

    let created = ServerEvent::RoomCreated {
        player_x_id,
        room_id: room_id.clone(),
    };
    if socket.send(created).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Ok(ClientEvent::SetName(name)) => {
                    with_room(&room_id, |room| room.set_name("X", name));
                }
                Ok(_) => {}
                Err(_) => break,
            },
            Some(event) = rx.recv() => {
                if socket.send(event).await.is_err() {
                    break;
                }
            }
        }
    }
}
