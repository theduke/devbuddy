use anyhow::Result;
use std::sync::Arc;

#[cfg(all(feature = "desktop", target_os = "linux"))]
mod linux;
#[cfg(all(feature = "desktop", not(target_os = "linux")))]
mod sdk;

#[cfg(all(feature = "desktop", target_os = "linux"))]
pub use linux::LinuxNotifier;
#[cfg(all(feature = "desktop", not(target_os = "linux")))]
pub use sdk::SdkNotifier;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NotificationOptions {
    pub key: Option<String>,
    pub actions: Vec<NotificationAction>,
    pub wait_for_dismiss: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NotifierSupport {
    pub updates: bool,
    pub actions: bool,
    pub dismissals: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationEvent {
    Sent,
    Action(String),
    Closed,
}

pub trait Notifier: Send + Sync {
    fn support(&self) -> NotifierSupport {
        NotifierSupport::default()
    }

    fn notify(&self, app_name: &str, summary: &str, body: &str) -> Result<()>;

    fn notify_with_options(
        &self,
        app_name: &str,
        summary: &str,
        body: &str,
        options: NotificationOptions,
    ) -> Result<NotificationEvent> {
        let _ = options;
        self.notify(app_name, summary, body)?;
        Ok(NotificationEvent::Sent)
    }

    fn update(&self, key: &str, app_name: &str, summary: &str, body: &str) -> Result<()> {
        let _ = key;
        self.notify(app_name, summary, body)
    }
}

pub type DynNotifier = Arc<dyn Notifier>;

pub fn build_notifier() -> DynNotifier {
    #[cfg(all(feature = "desktop", target_os = "linux"))]
    {
        Arc::new(LinuxNotifier::default())
    }

    #[cfg(all(feature = "desktop", not(target_os = "linux")))]
    {
        Arc::new(SdkNotifier)
    }

    #[cfg(not(feature = "desktop"))]
    {
        Arc::new(NoopNotifier)
    }
}

#[cfg(not(feature = "desktop"))]
struct NoopNotifier;

#[cfg(not(feature = "desktop"))]
impl Notifier for NoopNotifier {
    fn notify(&self, _app_name: &str, _summary: &str, _body: &str) -> Result<()> {
        Ok(())
    }
}
