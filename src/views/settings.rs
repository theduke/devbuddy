use dioxus::prelude::*;
use dioxus_bulma::{Container, Section};
use dioxus_sdk_notification::Notification as DesktopNotification;

#[component]
pub fn Settings() -> Element {
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
            }
        }
    }
}
