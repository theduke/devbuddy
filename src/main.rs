#![allow(non_snake_case)]

pub mod assets;
mod components;
mod notify;
mod route;
mod source;
mod store;

mod views;

use crate::assets::{install, BULMA_CSS, FAVICON, MAIN_CSS};
use dioxus::prelude::*;
use route::Route;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    install();

    let store = crate::store::build_store();
    use_context_provider(|| store.clone());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Title { "GitHub review requests" }
        document::Stylesheet { href: BULMA_CSS }
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
