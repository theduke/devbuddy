use anyhow::Result;
use std::sync::Arc;

#[cfg(feature = "desktop")]
mod sdk;

#[cfg(feature = "desktop")]
pub use sdk::SdkNotifier;

pub trait Notifier: Send + Sync {
    fn notify(&self, app_name: &str, summary: &str, body: &str) -> Result<()>;
}

pub type DynNotifier = Arc<dyn Notifier>;

pub fn build_notifier() -> DynNotifier {
    #[cfg(feature = "desktop")]
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
