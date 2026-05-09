//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component  to be used in our app.

pub mod github_config_form;
mod hero;
pub mod item;
#[allow(unused_imports)]
pub use hero::Hero;
