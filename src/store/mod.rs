pub mod types;

use async_trait::async_trait;

pub use types::Item;

#[async_trait]
pub trait Store: Send + Sync {
    async fn load_items(&self) -> anyhow::Result<Vec<Item>>;

    async fn store_items(&self, items: Vec<Item>) -> anyhow::Result<()>;
}
