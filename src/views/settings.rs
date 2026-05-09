use crate::components::github_config_form::GithubConfigForm;
use crate::source::github::{resolve_github_token_with_source, GithubTokenSource};
use crate::store::types::Config;
use crate::store::use_store;
use dioxus::prelude::*;
use dioxus_bulma::{Container, Section};
use dioxus_sdk_notification::Notification as DesktopNotification;

#[component]
pub fn Settings() -> Element {
    let store = use_store();
    let config = use_resource(move || {
        let store = store.clone();
        async move { store.load_config().await }
    });

    let send_test_notification = move |_| {
        let mut notification = DesktopNotification::new();
        notification
            .app_name("Devbuddy")
            .summary("Test notification")
            .body("This is a sample notification from the settings page.");

        if let Err(err) = notification.show() {
            eprintln!("failed to show desktop notification: {err}");
        }
    };

    let github_token_source = match &*config.value().read_unchecked() {
        None => "Loading GitHub config...".to_string(),
        Some(Ok(config)) => github_token_source_label(config),
        Some(Err(err)) => format!("Failed to load GitHub config: {err}"),
    };

    rsx! {
        Section {
            Container {
                h2 { class: "title is-2", "Settings" }

                div { class: "content",
                    h3 { class: "title is-4", "Notifications" }
                    button {
                        class: "button is-link",
                        onclick: send_test_notification,
                        "Send test notification"
                    }
                }

                div { class: "content",
                    h3 { class: "title is-4", "GitHub" }
                    p {
                        strong { "Token source: " }
                        "{github_token_source}"
                    }
                }

                GithubConfigForm {}
            }
        }
    }
}

fn github_token_source_label(config: &Config) -> String {
    if config.github_token.is_some() {
        return "custom config".to_string();
    }

    match resolve_github_token_with_source() {
        Ok((_token, GithubTokenSource::EnvironmentVariable(source))) => {
            format!("auto-detected from environment variable {source}")
        }
        Ok((_token, GithubTokenSource::GithubCli)) => {
            "auto-detected from GitHub CLI (gh auth token)".to_string()
        }
        Ok((_token, GithubTokenSource::CustomConfig)) => "custom config".to_string(),
        Err(err) => format!("not configured; auto-detection failed: {err}"),
    }
}
