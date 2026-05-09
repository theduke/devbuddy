#![allow(non_snake_case)]

mod components;
mod route;
mod source;
mod store;
mod views;

use dioxus::prelude::*;
use route::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Title { "GitHub review requests" }
        dioxus_bulma::embed::StylesheetBulma {}
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
