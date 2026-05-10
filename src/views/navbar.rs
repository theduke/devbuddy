use crate::{
    assets::{FAVICON, GEAR},
    Route,
};
use dioxus::prelude::*;
use dioxus_bulma::{Container, Section};

#[component]
pub fn Navbar() -> Element {
    let route = use_route::<Route>();

    let settings_class = if matches!(route, Route::Settings {}) {
        "navbar-item is-active p-1"
    } else {
        "navbar-item p-1"
    };

    rsx! {
        nav {
            class: "navbar is-fixed-top has-shadow is-white",
            role: "navigation",
            aria_label: "main navigation",
            style: "display: flex; align-items: center;",
            Container {
                style: "display: flex; align-items: center; justify-content: space-between; width: 100%;",
                div {
                    class: "navbar-brand",
                    Link {
                        to: Route::Home {},
                        class: "navbar-item",
                        img {
                            src: FAVICON,
                            alt: "DevBuddy logo",
                        }
                        span { class: "ml-2 has-text-weight-semibold", "DevBuddy" }
                    }
                }

                div {
                    class: "navbar-menu is-active",
                    style: "display: flex; flex-grow: 0; background: transparent; box-shadow: none;",
                    div {
                        class: "navbar-end",
                        Link {
                            to: Route::Settings {},
                            class: settings_class,
                            title: "Settings",
                            aria_label: "Settings",
                            span { class: "icon is-medium", img { src: GEAR, alt: "" } }
                            span { class: "is-sr-only", "Settings" }
                        }
                    }
                }
            }
        }

        Section {
            style: "margin-top: 1.1rem;",
            Outlet::<Route> {}
        }
    }
}
