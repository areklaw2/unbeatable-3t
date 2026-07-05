use api::{ServerEvent, join_room_ws};
use dioxus::fullstack::WebSocketOptions;
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};

use crate::Route;
use crate::components::{Button, ButtonVariant, Input};
use crate::state::{APP_STATE, AppStateStoreExt};

const JOIN_ROOM_CSS: Asset = asset!("/assets/styling/join_room.css");

fn sanitize_code(raw: &str) -> String {
    raw.chars()
        .flat_map(|c| c.to_uppercase())
        .filter(|c| c.is_ascii_alphanumeric())
        .take(4)
        .collect()
}

#[component]
pub fn JoinRoom(code: Option<String>) -> Element {
    let nav = use_navigator();
    let mut room_code = use_signal(move || sanitize_code(&code.clone().unwrap_or_default()));
    let store = APP_STATE.resolve();
    let mut name = store.name_o();
    let mut error = use_signal(|| None::<String>);
    let mut joining = use_signal(|| false);

    let join = move |_| {
        let code_value = room_code.cloned();
        let name_value = name.cloned();

        spawn(async move {
            error.set(None);
            joining.set(true);

            let socket =
                match join_room_ws(code_value, Some(name_value), WebSocketOptions::new()).await {
                    Ok(socket) => socket,
                    Err(_) => {
                        error.set(Some("Couldn't connect. Try again.".to_string()));
                        joining.set(false);
                        return;
                    }
                };

            match socket.recv().await {
                Ok(ServerEvent::RoomJoined { player_o_id, .. }) => {
                    let _ = LocalStorage::set("player_id", player_o_id);
                    nav.push(Route::Game {});
                }
                Ok(ServerEvent::RoomNotFound) => {
                    error.set(Some("Room not found.".to_string()));
                    joining.set(false);
                }
                _ => {
                    error.set(Some("Something went wrong. Try again.".to_string()));
                    joining.set(false);
                }
            }
        });
    };

    rsx! {
        document::Link { rel: "stylesheet", href: JOIN_ROOM_CSS }

        div { class: "card screen",
            div { class: "heading", "Join a room" }

            div { class: "section-label", "ROOM CODE" }
            div { class: "code-input",
                Input {
                    value: "{room_code}",
                    placeholder: "XXXX",
                    maxlength: 4,
                    oninput: move |e: FormEvent| room_code.set(sanitize_code(&e.value())),
                }
            }

            div { class: "section-label", "YOUR NAME" }
            div { class: "name-input",
                Input {
                    value: "{name}",
                    placeholder: "Player 2",
                    maxlength: 12,
                    oninput: move |e: FormEvent| name.set(e.value()),
                }
            }

            if let Some(message) = error() {
                div { class: "error-text", "{message}" }
            }

            Button {
                class: "join-button",
                variant: ButtonVariant::Primary,
                disabled: joining() || room_code().len() < 4,
                onclick: join,
                if joining() {
                    "Joining…"
                } else {
                    "Join game"
                }
            }

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
