use dioxus::prelude::*;

use crate::Route;
use crate::components::{Button, ButtonVariant};

const MULTIPLAYER_CSS: Asset = asset!("/assets/styling/multiplayer.css");

#[component]
pub fn Multiplayer() -> Element {
    let nav = use_navigator();

    rsx! {
        document::Link { rel: "stylesheet", href: MULTIPLAYER_CSS }

        div { class: "card screen",
            div { class: "heading", "Two players" }
            div { class: "subtitle", "Play with a friend on another device." }

            div { class: "mp-actions",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| {
                        nav.push(Route::CreateRoom {});
                    },
                    "Create a room"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        nav.push(Route::JoinRoom {});
                    },
                    "Join with a code"
                }
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
