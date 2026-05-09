use anyhow::{anyhow, Result};
use dioxus_sdk_notification::Notification as DesktopNotification;

use super::Notifier;

pub struct SdkNotifier;

impl Notifier for SdkNotifier {
    fn notify(&self, app_name: &str, summary: &str, body: &str) -> Result<()> {
        let mut notification = DesktopNotification::new();
        notification
            .app_name(app_name.to_string())
            .summary(summary.to_string())
            .body(body.to_string())
            .show()
            .map_err(|err| anyhow!(err.to_string()))?;
        Ok(())
    }
}
