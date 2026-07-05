use api::{ClientEvent, ServerEvent, create_room_ws};
use dioxus::fullstack::{WebSocketOptions, use_websocket};
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};

use crate::Route;
use crate::components::Input;
use crate::state::{APP_STATE, AppStateStoreExt};

const CREATE_ROOM_CSS: Asset = asset!("/assets/styling/create_room.css");

#[component]
pub fn CreateRoom() -> Element {
    let nav = use_navigator();
    let store = APP_STATE.resolve();
    let mut name = store.name_x();
    let mut room_code = use_signal(|| None::<String>);

    let mut socket =
        use_websocket(move || create_room_ws(Some(name.peek().clone()), WebSocketOptions::new()));

    use_effect(move || {
        let current_name = name.cloned();
        spawn(async move {
            let _ = socket.send(ClientEvent::SetName(current_name)).await;
        });
    });

    use_future(move || async move {
        loop {
            _ = socket.connect().await;

            while let Ok(msg) = socket.recv().await {
                match msg {
                    ServerEvent::RoomCreated {
                        player_x_id,
                        room_id,
                    } => {
                        let _ = LocalStorage::set("player_id", player_x_id);
                        room_code.set(Some(room_id));
                    }
                    ServerEvent::PlayerJoined { .. } => {
                        nav.push(Route::Game {});
                    }
                    _ => {}
                }
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CREATE_ROOM_CSS }

        div { class: "card screen",
            div { class: "heading", "Create a room" }

            div { class: "section-label", "YOUR NAME" }
            div { class: "name-input",
                Input {
                    value: "{name}",
                    placeholder: "Player",
                    maxlength: 12,
                    oninput: move |e: FormEvent| name.set(e.value()),
                }
            }

            div { class: "section-label", "ROOM CODE" }
            div { class: "code-row",
                if let Some(code) = room_code() {
                    for ch in code.chars() {
                        div { class: "code-box", "{ch}" }
                    }
                } else {
                    div { class: "code-box", "…" }
                }
            }

            div { class: "section-label", "SHAREABLE LINK" }
            div { class: "share-link-row",
                span { class: "share-link-text", "unbeatable3t.gg/#{room_code().unwrap_or_default()}" }
                button { class: "copy-button", "Copy" }
            }

            div { class: "waiting-text", "Waiting for player 2…" }

            button {
                class: "back-link",
                onclick: move |_| {
                    nav.push(Route::Multiplayer {});
                },
                "← Back"
            }
        }
    }
}
