pub mod fs;
pub mod types;

use async_trait::async_trait;
use dioxus::prelude::*;
use std::sync::Arc;

#[allow(unused_imports)]
pub use fs::FsStore;
pub use types::{Config, Item};

pub type DynStore = Arc<dyn Store>;

pub fn use_store() -> DynStore {
    use_context::<DynStore>()
}

#[allow(dead_code)]
#[async_trait]
pub trait Store: Send + Sync {
    async fn load_config(&self) -> anyhow::Result<Config>;

    async fn store_config(&self, config: Config) -> anyhow::Result<()>;

    async fn load_items(&self) -> anyhow::Result<Vec<Item>>;

    async fn store_items(&self, items: Vec<Item>) -> anyhow::Result<()>;
}

pub fn build_store() -> DynStore {
    #[cfg(any(feature = "desktop", feature = "native"))]
    {
        let s: DynStore = Arc::new(fs::FsStore::new(None));
        s
    }

    #[cfg(not(any(feature = "desktop", feature = "native")))]
    {
        panic!("no store implemented yet on web targets")
    }
}
