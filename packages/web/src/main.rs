use dioxus::prelude::*;

use views::{SinglePlayer, Title};

mod components;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Title {},
    #[route("/single")]
    SinglePlayer {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const FONT_CSS: &str =
    "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: FONT_CSS }
        document::Stylesheet { href: THEME_CSS }
        document::Stylesheet { href: MAIN_CSS }

        div { class: "app-shell", Router::<Route> {} }
    }
}
