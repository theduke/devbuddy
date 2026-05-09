use crate::Route;
use dioxus::prelude::*;
use dioxus_bulma::{Container, Section};

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        Section {
            Container {
                div {
                    id: "blog",
                    class: "content",

                    h1 { "This is blog #{id}!" }
                    p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

                    div { class: "buttons",
                        Link {
                            to: Route::Blog { id: id - 1 },
                            class: "button is-link is-light",
                            "Previous"
                        }
                        Link {
                            to: Route::Blog { id: id + 1 },
                            class: "button is-link is-light",
                            "Next"
                        }
                    }
                }
            }
        }
    }
}
