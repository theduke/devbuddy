#![allow(dead_code)]

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Item, Store};

const APP_NAME: &str = "devbuddy";
const ITEMS_FILE_NAME: &str = "items.jsonl";
const ITEMS_TMP_FILE_NAME: &str = "items.jsonl.tmp";

#[derive(Clone, Debug)]
pub struct FsStore {
    storage_dir: PathBuf,
}

impl FsStore {
    pub fn new(storage_dir: Option<PathBuf>) -> Self {
        Self {
            storage_dir: storage_dir.unwrap_or_else(default_storage_dir),
        }
    }

    pub fn storage_dir(&self) -> &Path {
        &self.storage_dir
    }

    fn items_path(&self) -> PathBuf {
        self.storage_dir.join(ITEMS_FILE_NAME)
    }

    fn temp_items_path(&self) -> PathBuf {
        self.storage_dir.join(ITEMS_TMP_FILE_NAME)
    }
}

#[async_trait]
impl Store for FsStore {
    async fn load_items(&self) -> Result<Vec<Item>> {
        let items_path = self.items_path();
        let file = match File::open(&items_path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to open store file at {}", items_path.display())
                });
            }
        };

        let reader = BufReader::new(file);
        let mut items = Vec::new();

        for (line_idx, line) in reader.lines().enumerate() {
            let line = line.with_context(|| {
                format!(
                    "failed to read line {} from store file {}",
                    line_idx + 1,
                    items_path.display()
                )
            })?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let item = serde_json::from_str::<Item>(line).with_context(|| {
                format!(
                    "failed to parse item on line {} from store file {}",
                    line_idx + 1,
                    items_path.display()
                )
            })?;
            items.push(item);
        }

        Ok(items)
    }

    async fn store_items(&self, items: Vec<Item>) -> Result<()> {
        fs::create_dir_all(&self.storage_dir).with_context(|| {
            format!(
                "failed to create store directory {}",
                self.storage_dir.display()
            )
        })?;

        let items_path = self.items_path();
        let temp_path = self.temp_items_path();

        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)
            .with_context(|| format!("failed to create temp store file {}", temp_path.display()))?;

        for item in items {
            serde_json::to_writer(&mut file, &item).with_context(|| {
                format!("failed to serialize item for {}", items_path.display())
            })?;
            file.write_all(b"\n")
                .with_context(|| format!("failed to write item to {}", temp_path.display()))?;
        }

        file.sync_all()
            .with_context(|| format!("failed to flush temp store file {}", temp_path.display()))?;
        drop(file);

        if items_path.exists() {
            fs::remove_file(&items_path).with_context(|| {
                format!(
                    "failed to clear existing store file {}",
                    items_path.display()
                )
            })?;
        }

        fs::rename(&temp_path, &items_path).with_context(|| {
            format!(
                "failed to replace store file {} with {}",
                items_path.display(),
                temp_path.display()
            )
        })?;

        Ok(())
    }
}

fn default_storage_dir() -> PathBuf {
    platform_data_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join(APP_NAME)
}

fn platform_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(dir) = std::env::var_os("XDG_DATA_HOME") {
            return Some(PathBuf::from(dir));
        }

        if let Some(home) = std::env::var_os("HOME") {
            return Some(PathBuf::from(home).join(".local/share"));
        }

        None
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support"))
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::types::{
        GithubReviewRequestItem, GithubUserPullRequestItem, ItemKind, PullRequestCiStatus,
        PullRequestReviewDecision,
    };
    use futures::executor::block_on;
    use std::time::{SystemTime, UNIX_EPOCH};
    use time::OffsetDateTime;

    #[test]
    fn round_trips_items() {
        let store = FsStore::new(Some(unique_temp_dir()));
        let items = vec![
            Item {
                kind: ItemKind::GithubReviewRequest(GithubReviewRequestItem {
                    owner: "acme".to_string(),
                    repo: "widgets".to_string(),
                    number: 7,
                    title: "Review me".to_string(),
                    author: "alice".to_string(),
                    html_url: "https://example.com/review".to_string(),
                    opened_at: timestamp(1),
                    last_pushed_at: Some(timestamp(2)),
                    updated_at: timestamp(3),
                    requested_at: Some(timestamp(4)),
                }),
                retrieved_at: timestamp(5),
                ignore: true,
                ignore_until: Some(timestamp(6)),
            },
            Item {
                kind: ItemKind::GithubUserPullRequest(GithubUserPullRequestItem {
                    owner: "acme".to_string(),
                    repo: "widgets".to_string(),
                    number: 8,
                    title: "Ship it".to_string(),
                    html_url: "https://example.com/open".to_string(),
                    opened_at: timestamp(7),
                    last_pushed_at: None,
                    review_decision: PullRequestReviewDecision::Approved,
                    ci_status: PullRequestCiStatus::Success,
                    last_review_comment_at: Some(timestamp(8)),
                    last_changes_requested_at: None,
                    last_approved_at: Some(timestamp(9)),
                    last_ci_failure_at: None,
                    last_ci_success_at: Some(timestamp(10)),
                    last_ci_started_at: Some(timestamp(11)),
                }),
                retrieved_at: timestamp(12),
                ignore: false,
                ignore_until: None,
            },
        ];

        block_on(async {
            store.store_items(items.clone()).await.unwrap();
            let loaded = store.load_items().await.unwrap();
            assert_eq!(loaded, items);
        });

        let _ = fs::remove_dir_all(store.storage_dir());
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("devbuddy-fsstore-{}-{}", std::process::id(), nanos))
    }

    fn timestamp(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(seconds).expect("valid timestamp")
    }
}
