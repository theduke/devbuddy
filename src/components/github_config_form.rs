use crate::source::github::GithubClient;
use crate::store::types::Config;
use crate::store::use_store;
use dioxus::prelude::*;
use futures::StreamExt;

#[component]
pub fn GithubConfigForm() -> Element {
    let store = use_store();
    let mut github_token = use_signal(String::new);
    let mut status = use_signal(|| None::<String>);
    let saving = use_signal(|| false);

    let save_token = use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        let store = store.clone();
        let mut status = status;
        let mut saving = saving;

        async move {
            while let Some(raw_token) = rx.next().await {
                let token = raw_token.trim().to_string();
                saving.set(true);
                status.set(None);

                if token.is_empty() {
                    status.set(Some("GitHub token cannot be empty".to_string()));
                    saving.set(false);
                    continue;
                }

                match GithubClient::with_token(token.clone())
                    .validate_token()
                    .await
                {
                    Ok(login) => {
                        let mut config: Config = match store.load_config().await {
                            Ok(config) => config,
                            Err(err) => {
                                status.set(Some(format!("Failed to load existing config: {err}")));
                                saving.set(false);
                                continue;
                            }
                        };

                        config.github_token = Some(token);

                        match store.store_config(config).await {
                            Ok(()) => {
                                status.set(Some(format!("Saved valid token for @{login}")));
                            }
                            Err(err) => {
                                status.set(Some(format!("Failed to save config: {err}")));
                            }
                        }
                    }
                    Err(err) => {
                        status.set(Some(format!("Token validation failed: {err}")));
                    }
                }

                saving.set(false);
            }
        }
    });

    rsx! {
        div { class: "content",
            h3 { class: "title is-4", "Custom GitHub token" }
            p { class: "help mb-4",
                "Validate a personal access token before saving it to the local config."
            }

            div { class: "field",
                label { class: "label", "Token" }
                div { class: "control",
                    input {
                        class: "input",
                        r#type: "password",
                        autocomplete: "off",
                        placeholder: "Paste GitHub token",
                        value: github_token(),
                        oninput: move |event| {
                            github_token.set(event.value());
                            status.set(None);
                        },
                    }
                }
            }

            div { class: "field is-grouped is-grouped-multiline",
                div { class: "control",
                    button {
                        class: if saving() { "button is-link is-loading" } else { "button is-link" },
                        disabled: saving(),
                        onclick: move |_| save_token.send(github_token()),
                        "Save token"
                    }
                }
            }

            if let Some(status) = status() {
                p { class: "help", "{status}" }
            }
        }
    }
}
