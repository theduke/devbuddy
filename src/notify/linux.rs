use anyhow::{anyhow, Result};
use notify_rust::{Notification, NotificationHandle};
use std::{collections::HashMap, sync::Mutex};

use super::{
    NotificationAction, NotificationEvent, NotificationOptions, Notifier, NotifierSupport,
};

#[derive(Default)]
pub struct LinuxNotifier {
    active: Mutex<HashMap<String, NotificationHandle>>,
}

impl LinuxNotifier {
    fn build_notification(app_name: &str, summary: &str, body: &str) -> Notification {
        let mut notification = Notification::new();
        notification.appname(app_name).summary(summary).body(body);
        notification
    }
}

impl Notifier for LinuxNotifier {
    fn support(&self) -> NotifierSupport {
        NotifierSupport {
            updates: true,
            actions: true,
            dismissals: true,
        }
    }

    fn notify(&self, app_name: &str, summary: &str, body: &str) -> Result<()> {
        let _ =
            self.notify_with_options(app_name, summary, body, NotificationOptions::default())?;
        Ok(())
    }

    fn notify_with_options(
        &self,
        app_name: &str,
        summary: &str,
        body: &str,
        options: NotificationOptions,
    ) -> Result<NotificationEvent> {
        if let Some(key) = options.key.as_ref() {
            if !options.wait_for_dismiss {
                let mut active = self.active.lock().unwrap();
                if let Some(handle) = active.get_mut(key) {
                    handle.appname(app_name);
                    handle.summary(summary);
                    handle.body(body);
                    for action in &options.actions {
                        handle.action(&action.id, &action.label);
                    }
                    handle.update().map_err(|err| anyhow!(err.to_string()))?;
                    return Ok(NotificationEvent::Sent);
                }
            }
        }

        let mut notification = Self::build_notification(app_name, summary, body);
        for NotificationAction { id, label } in &options.actions {
            notification.action(id, label);
        }

        let handle = notification
            .show()
            .map_err(|err| anyhow!(err.to_string()))?;

        if let Some(key) = options.key {
            if options.wait_for_dismiss {
                let mut event = NotificationEvent::Sent;
                handle.wait_for_action(|action| {
                    event = if action == "__closed" {
                        NotificationEvent::Closed
                    } else {
                        NotificationEvent::Action(action.to_string())
                    };
                });
                return Ok(event);
            }

            self.active.lock().unwrap().insert(key, handle);
            return Ok(NotificationEvent::Sent);
        }

        if options.wait_for_dismiss {
            let mut event = NotificationEvent::Sent;
            handle.wait_for_action(|action| {
                event = if action == "__closed" {
                    NotificationEvent::Closed
                } else {
                    NotificationEvent::Action(action.to_string())
                };
            });
            return Ok(event);
        }

        Ok(NotificationEvent::Sent)
    }

    fn update(&self, key: &str, app_name: &str, summary: &str, body: &str) -> Result<()> {
        let mut active = self.active.lock().unwrap();
        if let Some(handle) = active.get_mut(key) {
            handle.appname(app_name);
            handle.summary(summary);
            handle.body(body);
            handle.update().map_err(|err| anyhow!(err.to_string()))?;
            return Ok(());
        }

        drop(active);
        self.notify(app_name, summary, body)
    }
}
