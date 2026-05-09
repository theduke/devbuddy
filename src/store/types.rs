use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub retrieved_at: OffsetDateTime,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ignore: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_until: Option<OffsetDateTime>,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ItemKind {
    GithubReviewRequest(GithubReviewRequestItem),
    GithubUserPullRequest(GithubUserPullRequestItem),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubReviewRequestItem {
    pub owner: String,
    pub repo: String,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub html_url: String,
    pub opened_at: OffsetDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_pushed_at: Option<OffsetDateTime>,
    pub updated_at: OffsetDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubUserPullRequestItem {
    pub owner: String,
    pub repo: String,
    pub number: i64,
    pub title: String,
    pub html_url: String,
    pub opened_at: OffsetDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_ref_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_pushed_at: Option<OffsetDateTime>,
    pub review_decision: PullRequestReviewDecision,
    pub ci_status: PullRequestCiStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_review_comment_at: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_changes_requested_at: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_approved_at: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_ci_failure_at: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_ci_success_at: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_ci_started_at: Option<OffsetDateTime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PullRequestReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PullRequestCiStatus {
    Failed,
    InProgress,
    Success,
    Unknown,
}
