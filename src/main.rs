#![allow(non_snake_case)]

mod route;
mod source;
mod views;

use dioxus::prelude::*;
use route::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Title { "GitHub review requests" }
        dioxus_bulma::embed::StylesheetBulma {}
        Router::<Route> {}
    }
}
