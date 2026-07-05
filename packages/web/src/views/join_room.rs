use dioxus::prelude::*;

use crate::Route;
use crate::components::{Button, ButtonVariant, Input};
use crate::state::{APP_STATE, AppStateStoreExt, GameMode};

const JOIN_ROOM_CSS: Asset = asset!("/assets/styling/join_room.css");

fn sanitize_code(raw: &str) -> String {
    raw.chars()
        .flat_map(|c| c.to_uppercase())
        .filter(|c| c.is_ascii_alphanumeric())
        .take(4)
        .collect()
}

#[component]
pub fn JoinRoom() -> Element {
    let nav = use_navigator();
    let mut code = use_signal(String::new);
    let store = APP_STATE.resolve();
    let mut name = store.name_o();
    let mut game_mode = store.game_mode();

    rsx! {
        document::Link { rel: "stylesheet", href: JOIN_ROOM_CSS }

        div { class: "card screen",
            div { class: "heading", "Join a room" }

            div { class: "section-label", "ROOM CODE" }
            div { class: "code-input",
                Input {
                    value: "{code}",
                    placeholder: "XXXX",
                    maxlength: 4,
                    oninput: move |e: FormEvent| code.set(sanitize_code(&e.value())),
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

            Button {
                class: "join-button",
                variant: ButtonVariant::Primary,
                onclick: move |_| {
                    game_mode.set(GameMode::Multiple);
                    nav.push(Route::Game {});
                },
                "Join game"
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
