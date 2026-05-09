pub mod fs;
pub mod types;

use async_trait::async_trait;

#[allow(unused_imports)]
pub use fs::FsStore;
pub use types::{Config, Item};

#[allow(dead_code)]
#[async_trait]
pub trait Store: Send + Sync {
    async fn load_config(&self) -> anyhow::Result<Config>;

    async fn store_config(&self, config: Config) -> anyhow::Result<()>;

    async fn load_items(&self) -> anyhow::Result<Vec<Item>>;

    async fn store_items(&self, items: Vec<Item>) -> anyhow::Result<()>;
}
