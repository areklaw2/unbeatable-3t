use api::Mode;
use dioxus::prelude::*;

use crate::Route;
use crate::components::{Button, ButtonVariant, Input};
use crate::state::{APP_STATE, AppStateStoreExt};

const SINGLE_PLAYER_CSS: Asset = asset!("/assets/styling/single_player.css");

#[component]
pub fn SinglePlayer() -> Element {
    let nav = use_navigator();
    let store = APP_STATE.resolve();
    let mut name = store.name_x();
    let mut difficulty = store.difficulty();

    rsx! {
        document::Link { rel: "stylesheet", href: SINGLE_PLAYER_CSS }

        div { class: "card screen",
            div { class: "heading", "Single Player" }

            div { class: "section-label", "YOUR NAME" }
            div { class: "name-input",
                Input {
                    value: "{name}",
                    placeholder: "Player",
                    maxlength: 12,
                    oninput: move |e: FormEvent| name.set(e.value()),
                }
            }

            div { class: "section-label", "DIFFICULTY" }
            div { class: "difficulty-row",
                Button {
                    variant: if difficulty.cloned() == Mode::Easy { ButtonVariant::Primary } else { ButtonVariant::Outline },
                    onclick: move |_| difficulty.set(Mode::Easy),
                    "Easy"
                }
                Button {
                    variant: if difficulty.cloned() == Mode::Hard { ButtonVariant::Primary } else { ButtonVariant::Outline },
                    onclick: move |_| difficulty.set(Mode::Hard),
                    "Unbeatable"
                }
            }

            Button {
                class: "start-button",
                variant: ButtonVariant::Primary,
                onclick: move |_| {
                    nav.push(Route::Game { room_id: None });
                },
                "Start game"
            }

            button {
                class: "back-link",
                onclick: move |_| {
                    nav.push(Route::Title {});
                },
                "← Back"
            }
        }
    }
}
