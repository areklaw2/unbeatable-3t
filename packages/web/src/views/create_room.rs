use dioxus::prelude::*;

use crate::Route;
use crate::components::{Button, ButtonVariant, Input};
use crate::state::{APP_STATE, AppStateStoreExt};

const CREATE_ROOM_CSS: Asset = asset!("/assets/styling/create_room.css");

// Placeholder until room creation is wired to a real server-issued code.
const ROOM_CODE: &str = "8FQR";

#[component]
pub fn CreateRoom() -> Element {
    let nav = use_navigator();
    let store = APP_STATE.resolve();
    let mut name = store.name_x();

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
                for ch in ROOM_CODE.chars() {
                    div { class: "code-box", "{ch}" }
                }
            }

            div { class: "section-label", "SHAREABLE LINK" }
            div { class: "share-link-row",
                span { class: "share-link-text", "unbeatable3t.gg/#{ROOM_CODE}" }
                button { class: "copy-button", "Copy" }
            }

            div { class: "waiting-text", "Waiting for player 2…" }

            Button {
                class: "start-button",
                variant: ButtonVariant::Primary,
                onclick: move |_| {
                    nav.push(Route::Game {});
                },
                "Start game"
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
