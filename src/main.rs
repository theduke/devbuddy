#![allow(non_snake_case)]

mod components;
mod notify;
mod route;
mod source;
mod store;
mod views;

use dioxus::prelude::*;
use route::Route;
use std::sync::Arc;

use crate::store::{DynStore, FsStore};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let store: DynStore = Arc::new(FsStore::new(None));
    use_context_provider(|| store.clone());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Title { "GitHub review requests" }
        dioxus_bulma::embed::StylesheetBulma {}
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
