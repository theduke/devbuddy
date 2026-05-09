use crate::Route;
use dioxus::prelude::*;
use dioxus_bulma::{Container, Section};

#[component]
pub fn Navbar() -> Element {
    let route = use_route::<Route>();
    let mut menu_open = use_signal(|| false);

    let home_class = if matches!(route, Route::Home {}) {
        "navbar-item is-active"
    } else {
        "navbar-item"
    };

    let blog_class = if matches!(route, Route::Blog { .. }) {
        "navbar-item is-active"
    } else {
        "navbar-item"
    };

    let settings_class = if matches!(route, Route::Settings {}) {
        "navbar-item is-active"
    } else {
        "navbar-item"
    };

    rsx! {
        nav {
            class: "navbar is-fixed-top has-shadow is-white",
            role: "navigation",
            aria_label: "main navigation",
            Container {
                div {
                    class: "navbar-brand",
                    Link {
                        to: Route::Home {},
                        class: "navbar-item",
                        onclick: move |_| menu_open.set(false),
                        img {
                            src: asset!("/assets/favicon.ico"),
                            alt: "DevBuddy logo",
                        }
                        span { class: "ml-2 has-text-weight-semibold", "DevBuddy" }
                    }
                    button {
                        class: if menu_open() { "navbar-burger is-active" } else { "navbar-burger" },
                        aria_label: "menu",
                        aria_expanded: if menu_open() { "true" } else { "false" },
                        onclick: move |_| menu_open.set(!menu_open()),
                        span { aria_hidden: "true" }
                        span { aria_hidden: "true" }
                        span { aria_hidden: "true" }
                    }
                }

                div {
                    class: if menu_open() { "navbar-menu is-active" } else { "navbar-menu" },
                    div {
                        class: "navbar-start",
                        Link {
                            to: Route::Home {},
                            class: home_class,
                            onclick: move |_| menu_open.set(false),
                            "Home"
                        }
                        Link {
                            to: Route::Blog { id: 1 },
                            class: blog_class,
                            onclick: move |_| menu_open.set(false),
                            "Blog"
                        }
                    }

                    div {
                        class: "navbar-end",
                        Link {
                            to: Route::Settings {},
                            class: settings_class,
                            title: "Settings",
                            aria_label: "Settings",
                            onclick: move |_| menu_open.set(false),
                            span { class: "icon is-small", "⚙" }
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
