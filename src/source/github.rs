#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, env};
use time::OffsetDateTime;

pub const GITHUB_API_BASE_URL: &str = "https://api.github.com";
pub const GITHUB_REVIEW_REQUESTED_PRS_QUERY: &str = "is:pr review-requested:@me state:open";
pub const GITHUB_OPEN_PRS_QUERY: &str = "is:pr author:@me state:open";
pub const GITHUB_REVIEW_REQUESTED_PRS_GRAPHQL: &str = r#"query($query: String!, $after: String) {
  viewer {
    login
  }
  search(query: $query, type: ISSUE, first: 100, after: $after) {
    nodes {
      __typename
      ... on PullRequest {
        title
        url
        number
              createdAt
        state
        updatedAt
              commits(last: 1) {
                nodes {
                  commit {
                    committedDate
                  }
                }
              }
        repository {
          name
          owner {
            login
          }
        }
        author {
          login
        }
        timelineItems(first: 100, itemTypes: [REVIEW_REQUESTED_EVENT]) {
          nodes {
            __typename
            ... on ReviewRequestedEvent {
              createdAt
              requestedReviewer {
                __typename
                ... on User {
                  login
                }
              }
            }
          }
        }
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}"#;
pub const GITHUB_OPEN_PRS_GRAPHQL: &str = r#"query($query: String!, $after: String) {
  search(query: $query, type: ISSUE, first: 100, after: $after) {
    nodes {
      __typename
      ... on PullRequest {
        title
        url
        number
        createdAt
        updatedAt
        reviewDecision
        reviews(last: 20, states: [APPROVED, CHANGES_REQUESTED, COMMENTED]) {
          nodes {
            state
            submittedAt
          }
        }
        repository {
          name
          owner {
            login
          }
        }
        headRefName
        commits(last: 1) {
          nodes {
            commit {
              committedDate
              statusCheckRollup {
                state
                contexts(last: 100) {
                  nodes {
                    __typename
                    ... on CheckRun {
                      status
                      conclusion
                      completedAt
                      startedAt
                    }
                    ... on StatusContext {
                      state
                      createdAt
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}"#;

pub const GITHUB_VIEWER_GRAPHQL: &str = r#"query {
  viewer {
    login
  }
}"#;

pub const GITHUB_TOKEN_ENV_VARS: &[&str] = &[
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "GITHUB_API_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GithubTokenSource {
    CustomConfig,
    EnvironmentVariable(&'static str),
    GithubCli,
}

#[derive(Debug, Clone)]
pub struct GithubClient {
    token: String,
    base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestSummary {
    pub owner: String,
    pub repo: String,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub html_url: String,
    pub opened_at: OffsetDateTime,
    pub last_pushed_at: Option<OffsetDateTime>,
    pub updated_at: OffsetDateTime,
    pub requested_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPullRequestSummary {
    pub owner: String,
    pub repo: String,
    pub number: i64,
    pub title: String,
    pub html_url: String,
    pub opened_at: OffsetDateTime,
    pub head_ref_name: Option<String>,
    pub last_pushed_at: Option<OffsetDateTime>,
    pub review_decision: PullRequestReviewDecision,
    pub ci_status: PullRequestCiStatus,
    pub last_review_comment_at: Option<OffsetDateTime>,
    pub last_changes_requested_at: Option<OffsetDateTime>,
    pub last_approved_at: Option<OffsetDateTime>,
    pub last_ci_failure_at: Option<OffsetDateTime>,
    pub last_ci_success_at: Option<OffsetDateTime>,
    pub last_ci_started_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullRequestReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullRequestCiStatus {
    Success,
    Failed,
    InProgress,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GithubCiRunStatus {
    pub total_jobs: usize,
    pub in_progress_jobs: usize,
    pub failed_jobs: usize,
    pub succeeded_jobs: usize,
}

#[derive(Debug, Serialize)]
struct GithubGraphQLRequest<V> {
    query: String,
    variables: V,
}

#[derive(Debug, Serialize)]
struct GithubReviewRequestedPRsVariables {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<String>,
}

#[derive(Debug, Serialize)]
struct GithubOpenPullRequestsVariables {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<String>,
}

#[derive(Debug, Serialize)]
struct GithubEmptyVariables;

#[derive(Debug, Deserialize)]
struct GithubGraphQLResponse<T> {
    data: T,
    #[serde(default)]
    errors: Vec<GithubGraphQLError>,
}

#[derive(Debug, Deserialize)]
struct GithubGraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubGraphQLResponseData {
    viewer: GithubViewer,
    search: GithubSearchResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubOpenPullRequestsResponseData {
    search: GithubOpenPullRequestsSearchResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubViewerResponseData {
    viewer: GithubViewer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubViewer {
    login: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubSearchResult {
    nodes: Vec<GithubSearchNode>,
    page_info: GithubPageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubOpenPullRequestsSearchResult {
    nodes: Vec<GithubOpenPullRequestsSearchNode>,
    page_info: GithubPageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPageInfo {
    has_next_page: bool,
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubSearchNode {
    #[serde(rename = "__typename")]
    typename: String,
    title: Option<String>,
    url: Option<String>,
    number: Option<i64>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    created_at: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    updated_at: Option<OffsetDateTime>,
    repository: Option<GithubRepository>,
    author: Option<GithubPullRequestAuthor>,
    #[serde(default)]
    commits: GithubPullRequestCommitConn,
    #[serde(default)]
    timeline_items: GithubTimelineItemConn,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubOpenPullRequestsSearchNode {
    #[serde(rename = "__typename")]
    typename: String,
    title: Option<String>,
    url: Option<String>,
    number: Option<i64>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    created_at: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    updated_at: Option<OffsetDateTime>,
    review_decision: Option<GithubReviewDecision>,
    #[serde(default)]
    reviews: GithubPullRequestReviewConn,
    repository: Option<GithubRepository>,
    head_ref_name: Option<String>,
    #[serde(default)]
    commits: GithubPullRequestCommitConn,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestAuthor {
    login: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubTimelineItemConn {
    #[serde(default)]
    nodes: Vec<GithubTimelineItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubTimelineItem {
    #[serde(rename = "__typename")]
    typename: String,
    #[serde(default, with = "time::serde::rfc3339::option")]
    created_at: Option<OffsetDateTime>,
    requested_reviewer: Option<GithubRequestedReviewer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRequestedReviewer {
    #[serde(rename = "__typename")]
    typename: String,
    login: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestCommitConn {
    #[serde(default)]
    nodes: Vec<GithubPullRequestCommitNode>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestReviewConn {
    #[serde(default)]
    nodes: Vec<GithubPullRequestReview>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestReview {
    state: Option<GithubPullRequestReviewState>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    submitted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestCommitNode {
    commit: Option<GithubCommit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubCommit {
    #[serde(default, with = "time::serde::rfc3339::option")]
    committed_date: Option<OffsetDateTime>,
    status_check_rollup: Option<GithubStatusCheckRollup>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubStatusCheckRollup {
    state: Option<GithubStatusState>,
    #[serde(default)]
    contexts: GithubStatusCheckContextConn,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubStatusCheckContextConn {
    #[serde(default)]
    nodes: Vec<GithubStatusCheckContextNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubStatusCheckContextNode {
    #[serde(rename = "__typename")]
    typename: String,
    status: Option<GithubCheckStatus>,
    conclusion: Option<GithubCheckConclusionState>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    completed_at: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    started_at: Option<OffsetDateTime>,
    state: Option<GithubStatusState>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    created_at: Option<OffsetDateTime>,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GithubStatusState {
    Expected,
    Error,
    Failure,
    Pending,
    Success,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GithubReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GithubPullRequestReviewState {
    Approved,
    ChangesRequested,
    Commented,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GithubCheckStatus {
    Completed,
    InProgress,
    Pending,
    Queued,
    Requested,
    Waiting,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GithubCheckConclusionState {
    ActionRequired,
    Cancelled,
    Failure,
    Neutral,
    Skipped,
    Stale,
    StartupFailure,
    Success,
    TimedOut,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepository {
    name: Option<String>,
    owner: Option<GithubRepositoryOwner>,
}

#[derive(Debug, Deserialize)]
struct GithubWorkflowRunsResponse {
    #[serde(default)]
    workflow_runs: Vec<GithubWorkflowRunSummary>,
}

#[derive(Debug, Deserialize)]
struct GithubWorkflowRunSummary {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct GithubWorkflowRunJobsResponse {
    total_count: usize,
    jobs: Vec<GithubWorkflowRunJob>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubWorkflowRunJob {
    status: Option<GithubWorkflowRunJobStatus>,
    conclusion: Option<GithubWorkflowRunJobConclusion>,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GithubWorkflowRunJobStatus {
    Completed,
    InProgress,
    Pending,
    Queued,
    Requested,
    Waiting,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GithubWorkflowRunJobConclusion {
    ActionRequired,
    Cancelled,
    Failure,
    Neutral,
    Skipped,
    Stale,
    StartupFailure,
    Success,
    TimedOut,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepositoryOwner {
    login: Option<String>,
}

impl GithubClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            token: resolve_github_token()?,
            base_url: GITHUB_API_BASE_URL.to_string(),
        })
    }

    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            base_url: GITHUB_API_BASE_URL.to_string(),
        }
    }

    pub async fn pull_requests_requested_for_review(&self) -> Result<Vec<PullRequestSummary>> {
        if self.token.trim().is_empty() {
            return Err(anyhow!("github token is empty"));
        }

        let mut results = Vec::new();
        let mut after: Option<String> = None;

        loop {
            let page = self.search_review_requested_prs(after.clone()).await?;

            for node in page.search.nodes {
                if node.typename != "PullRequest" {
                    continue;
                }

                let owner = node
                    .repository
                    .as_ref()
                    .and_then(|repo| repo.owner.as_ref())
                    .and_then(|owner| owner.login.clone());
                let repo = node.repository.as_ref().and_then(|repo| repo.name.clone());
                let author = node.author.as_ref().and_then(|author| author.login.clone());
                let (
                    Some(title),
                    Some(url),
                    Some(number),
                    Some(opened_at),
                    Some(updated_at),
                    Some(owner),
                    Some(repo),
                    Some(author),
                ) = (
                    node.title,
                    node.url,
                    node.number,
                    node.created_at,
                    node.updated_at,
                    owner,
                    repo,
                    author,
                )
                else {
                    continue;
                };

                let requested_at =
                    requested_at_for_viewer(&node.timeline_items.nodes, &page.viewer.login);
                let last_pushed_at = node
                    .commits
                    .nodes
                    .first()
                    .and_then(|node| node.commit.as_ref())
                    .and_then(|commit| commit.committed_date);
                results.push(PullRequestSummary {
                    owner,
                    repo,
                    number,
                    title,
                    author,
                    html_url: url,
                    opened_at,
                    last_pushed_at,
                    updated_at,
                    requested_at,
                });
            }

            if !page.search.page_info.has_next_page {
                break;
            }

            match page.search.page_info.end_cursor {
                Some(cursor) if !cursor.is_empty() => after = Some(cursor),
                _ => break,
            }
        }

        results.sort_by(|a, b| {
            let requested_order = match (a.requested_at, b.requested_at) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            };

            if requested_order == Ordering::Equal {
                a.updated_at.cmp(&b.updated_at)
            } else {
                requested_order
            }
        });

        Ok(results)
    }

    pub async fn validate_token(&self) -> Result<String> {
        if self.token.trim().is_empty() {
            return Err(anyhow!("github token is empty"));
        }

        let response: GithubViewerResponseData = do_github_graphql_request(
            &self.base_url,
            &self.token,
            GITHUB_VIEWER_GRAPHQL,
            GithubEmptyVariables,
        )
        .await?;

        Ok(response.viewer.login)
    }

    pub async fn ci_run_status(
        &self,
        owner: &str,
        repo: &str,
        run_id: i64,
    ) -> Result<GithubCiRunStatus> {
        if self.token.trim().is_empty() {
            return Err(anyhow!("github token is empty"));
        }
        if owner.trim().is_empty() {
            return Err(anyhow!("github owner is empty"));
        }
        if repo.trim().is_empty() {
            return Err(anyhow!("github repo is empty"));
        }

        let mut status = GithubCiRunStatus::default();
        let mut page = 1usize;

        loop {
            let response: GithubWorkflowRunJobsResponse = do_github_rest_request(
                &self.base_url,
                &self.token,
                &format!(
                    "/repos/{owner}/{repo}/actions/runs/{run_id}/jobs?per_page=100&page={page}"
                ),
            )
            .await?;

            if page == 1 {
                status.total_jobs = response.total_count;
            }

            if response.jobs.is_empty() {
                break;
            }

            for job in response.jobs {
                match (job.status, job.conclusion) {
                    (
                        Some(GithubWorkflowRunJobStatus::Completed),
                        Some(
                            GithubWorkflowRunJobConclusion::Success
                            | GithubWorkflowRunJobConclusion::Neutral
                            | GithubWorkflowRunJobConclusion::Skipped,
                        ),
                    ) => {
                        status.succeeded_jobs += 1;
                    }
                    (
                        Some(GithubWorkflowRunJobStatus::Completed),
                        Some(
                            GithubWorkflowRunJobConclusion::ActionRequired
                            | GithubWorkflowRunJobConclusion::Cancelled
                            | GithubWorkflowRunJobConclusion::Failure
                            | GithubWorkflowRunJobConclusion::Stale
                            | GithubWorkflowRunJobConclusion::StartupFailure
                            | GithubWorkflowRunJobConclusion::TimedOut,
                        ),
                    ) => {
                        status.failed_jobs += 1;
                    }
                    _ => {
                        status.in_progress_jobs += 1;
                    }
                }
            }

            let counted_jobs = status.in_progress_jobs + status.failed_jobs + status.succeeded_jobs;
            if counted_jobs >= status.total_jobs {
                break;
            }

            page += 1;
        }

        Ok(status)
    }

    pub async fn active_ci_run_status(
        &self,
        owner: &str,
        repo: &str,
        head_ref_name: &str,
    ) -> Result<Option<GithubCiRunStatus>> {
        if self.token.trim().is_empty() {
            return Err(anyhow!("github token is empty"));
        }
        if owner.trim().is_empty() {
            return Err(anyhow!("github owner is empty"));
        }
        if repo.trim().is_empty() {
            return Err(anyhow!("github repo is empty"));
        }
        if head_ref_name.trim().is_empty() {
            return Ok(None);
        }

        let response: GithubWorkflowRunsResponse = do_github_rest_request(
            &self.base_url,
            &self.token,
            &format!(
                "/repos/{owner}/{repo}/actions/runs?branch={}&status=in_progress&per_page=1",
                encode_query_component(head_ref_name)
            ),
        )
        .await?;

        let Some(run) = response.workflow_runs.first() else {
            return Ok(None);
        };

        Ok(Some(self.ci_run_status(owner, repo, run.id).await?))
    }

    pub async fn open_pull_requests_for_user(&self) -> Result<Vec<OpenPullRequestSummary>> {
        if self.token.trim().is_empty() {
            return Err(anyhow!("github token is empty"));
        }

        let mut results = Vec::new();
        let mut after: Option<String> = None;

        loop {
            let page = self.search_open_pull_requests(after.clone()).await?;

            for node in page.search.nodes {
                if node.typename != "PullRequest" {
                    continue;
                }

                let owner = node
                    .repository
                    .as_ref()
                    .and_then(|repo| repo.owner.as_ref())
                    .and_then(|owner| owner.login.clone());
                let repo = node.repository.as_ref().and_then(|repo| repo.name.clone());
                let (
                    Some(title),
                    Some(url),
                    Some(number),
                    Some(opened_at),
                    Some(owner),
                    Some(repo),
                ) = (
                    node.title,
                    node.url,
                    node.number,
                    node.created_at,
                    owner,
                    repo,
                )
                else {
                    continue;
                };

                let last_review_comment_at =
                    latest_review_at(&node.reviews.nodes, GithubPullRequestReviewState::Commented);
                let last_changes_requested_at = latest_review_at(
                    &node.reviews.nodes,
                    GithubPullRequestReviewState::ChangesRequested,
                );
                let last_approved_at =
                    latest_review_at(&node.reviews.nodes, GithubPullRequestReviewState::Approved);
                let last_pushed_at = node
                    .commits
                    .nodes
                    .first()
                    .and_then(|node| node.commit.as_ref())
                    .and_then(|commit| commit.committed_date);
                let ci_timestamps = latest_ci_timestamps(
                    node.commits
                        .nodes
                        .first()
                        .and_then(|node| node.commit.as_ref())
                        .and_then(|commit| commit.status_check_rollup.as_ref()),
                );
                let ci_status = node
                    .commits
                    .nodes
                    .first()
                    .and_then(|node| node.commit.as_ref())
                    .and_then(|commit| commit.status_check_rollup.as_ref())
                    .and_then(|rollup| rollup.state)
                    .map(Into::into)
                    .unwrap_or(PullRequestCiStatus::Unknown);

                results.push(OpenPullRequestSummary {
                    owner,
                    repo,
                    number,
                    title,
                    html_url: url,
                    opened_at,
                    head_ref_name: node.head_ref_name,
                    last_pushed_at,
                    review_decision: node
                        .review_decision
                        .map(Into::into)
                        .unwrap_or(PullRequestReviewDecision::ReviewRequired),
                    ci_status,
                    last_review_comment_at,
                    last_changes_requested_at,
                    last_approved_at,
                    last_ci_failure_at: ci_timestamps.failed_at,
                    last_ci_success_at: ci_timestamps.success_at,
                    last_ci_started_at: ci_timestamps.started_at,
                });
            }

            if !page.search.page_info.has_next_page {
                break;
            }

            match page.search.page_info.end_cursor {
                Some(cursor) if !cursor.is_empty() => after = Some(cursor),
                _ => break,
            }
        }

        results.sort_by(|a, b| match (a.last_pushed_at, b.last_pushed_at) {
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.opened_at.cmp(&a.opened_at),
        });

        Ok(results)
    }

    async fn search_review_requested_prs(
        &self,
        after: Option<String>,
    ) -> Result<GithubGraphQLResponseData> {
        let variables = GithubReviewRequestedPRsVariables {
            query: GITHUB_REVIEW_REQUESTED_PRS_QUERY.to_string(),
            after,
        };

        do_github_graphql_request(
            &self.base_url,
            &self.token,
            GITHUB_REVIEW_REQUESTED_PRS_GRAPHQL,
            variables,
        )
        .await
    }

    async fn search_open_pull_requests(
        &self,
        after: Option<String>,
    ) -> Result<GithubOpenPullRequestsResponseData> {
        let variables = GithubOpenPullRequestsVariables {
            query: GITHUB_OPEN_PRS_QUERY.to_string(),
            after,
        };

        do_github_graphql_request(
            &self.base_url,
            &self.token,
            GITHUB_OPEN_PRS_GRAPHQL,
            variables,
        )
        .await
    }
}

fn requested_at_for_viewer(
    requests: &[GithubTimelineItem],
    viewer_login: &str,
) -> Option<OffsetDateTime> {
    let mut newest: Option<OffsetDateTime> = None;

    for request in requests {
        if request.typename != "ReviewRequestedEvent" {
            continue;
        }
        let Some(reviewer) = request.requested_reviewer.as_ref() else {
            continue;
        };
        let Some(created_at) = request.created_at else {
            continue;
        };
        if reviewer.typename != "User" || reviewer.login.as_deref() != Some(viewer_login) {
            continue;
        }

        newest = Some(match newest {
            Some(current) if current >= created_at => current,
            _ => created_at,
        });
    }

    if newest.is_some() {
        return newest;
    }

    requests.iter().find_map(|request| request.created_at)
}

impl From<GithubReviewDecision> for PullRequestReviewDecision {
    fn from(value: GithubReviewDecision) -> Self {
        match value {
            GithubReviewDecision::Approved => Self::Approved,
            GithubReviewDecision::ChangesRequested => Self::ChangesRequested,
            GithubReviewDecision::ReviewRequired => Self::ReviewRequired,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct PullRequestCiTimestamps {
    failed_at: Option<OffsetDateTime>,
    success_at: Option<OffsetDateTime>,
    started_at: Option<OffsetDateTime>,
}

fn latest_review_at(
    reviews: &[GithubPullRequestReview],
    state: GithubPullRequestReviewState,
) -> Option<OffsetDateTime> {
    reviews
        .iter()
        .filter_map(|review| match (review.state, review.submitted_at) {
            (Some(review_state), Some(submitted_at)) if review_state == state => Some(submitted_at),
            _ => None,
        })
        .max()
}

fn latest_ci_timestamps(rollup: Option<&GithubStatusCheckRollup>) -> PullRequestCiTimestamps {
    let Some(rollup) = rollup else {
        return PullRequestCiTimestamps::default();
    };

    let mut timestamps = PullRequestCiTimestamps::default();

    for context in &rollup.contexts.nodes {
        match context.typename.as_str() {
            "CheckRun" => {
                let action_at = context.completed_at.or(context.started_at);
                match (context.status, context.conclusion) {
                    (
                        Some(GithubCheckStatus::Completed),
                        Some(GithubCheckConclusionState::Success),
                    ) => {
                        timestamps.success_at = latest_timestamp(timestamps.success_at, action_at);
                    }
                    (
                        Some(GithubCheckStatus::Completed),
                        Some(
                            GithubCheckConclusionState::ActionRequired
                            | GithubCheckConclusionState::Cancelled
                            | GithubCheckConclusionState::Failure
                            | GithubCheckConclusionState::StartupFailure
                            | GithubCheckConclusionState::TimedOut,
                        ),
                    ) => {
                        timestamps.failed_at = latest_timestamp(timestamps.failed_at, action_at);
                    }
                    (
                        Some(
                            GithubCheckStatus::InProgress
                            | GithubCheckStatus::Pending
                            | GithubCheckStatus::Queued
                            | GithubCheckStatus::Requested
                            | GithubCheckStatus::Waiting,
                        ),
                        _,
                    ) => {
                        timestamps.started_at =
                            latest_timestamp(timestamps.started_at, context.started_at);
                    }
                    _ => {}
                }
            }
            "StatusContext" => match context.state {
                Some(GithubStatusState::Success) => {
                    timestamps.success_at =
                        latest_timestamp(timestamps.success_at, context.created_at);
                }
                Some(GithubStatusState::Failure | GithubStatusState::Error) => {
                    timestamps.failed_at =
                        latest_timestamp(timestamps.failed_at, context.created_at);
                }
                Some(GithubStatusState::Expected | GithubStatusState::Pending) => {
                    timestamps.started_at =
                        latest_timestamp(timestamps.started_at, context.created_at);
                }
                None => {}
            },
            _ => {}
        }
    }

    timestamps
}

fn latest_timestamp(
    current: Option<OffsetDateTime>,
    candidate: Option<OffsetDateTime>,
) -> Option<OffsetDateTime> {
    match (current, candidate) {
        (Some(current), Some(candidate)) => Some(current.max(candidate)),
        (None, Some(candidate)) => Some(candidate),
        (Some(current), None) => Some(current),
        (None, None) => None,
    }
}

impl From<GithubStatusState> for PullRequestCiStatus {
    fn from(value: GithubStatusState) -> Self {
        match value {
            GithubStatusState::Success => Self::Success,
            GithubStatusState::Failure | GithubStatusState::Error => Self::Failed,
            GithubStatusState::Pending | GithubStatusState::Expected => Self::InProgress,
        }
    }
}

async fn execute_github_request(req: http::Request<String>) -> Result<http::Response<Vec<u8>>> {
    return super::http::execute_request(req).await;
}

async fn do_github_rest_request<T>(base_url: &str, token: &str, path: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let endpoint = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    );
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(endpoint)
        .header(http::header::ACCEPT, "application/vnd.github+json")
        .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(String::new())?;

    let response = execute_github_request(request).await?;
    let status = response.status();
    let body = response.into_body();

    if !status.is_success() {
        let body_text = String::from_utf8_lossy(&body).trim().to_string();
        return Err(anyhow!("github rest request failed: {status}: {body_text}"));
    }

    Ok(serde_json::from_slice(&body)?)
}

async fn do_github_graphql_request<T, V>(
    base_url: &str,
    token: &str,
    query: &str,
    variables: V,
) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
    V: Serialize,
{
    let endpoint = format!("{}/graphql", base_url.trim_end_matches('/'));
    let req_body = GithubGraphQLRequest {
        query: query.to_string(),
        variables,
    };
    let payload = serde_json::to_string(&req_body)?;

    let request = http::Request::builder()
        .method(http::Method::POST)
        .uri(endpoint)
        .header(http::header::ACCEPT, "application/json")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(payload)?;

    let response = execute_github_request(request).await?;
    let status = response.status();
    let body = response.into_body();

    if !status.is_success() {
        let body_text = String::from_utf8_lossy(&body).trim().to_string();
        return Err(anyhow!(
            "github graphql request failed: {status}: {body_text}"
        ));
    }

    let parsed: GithubGraphQLResponse<T> = serde_json::from_slice(&body)?;
    if let Some(err) = parsed.errors.into_iter().next() {
        return Err(anyhow!(err.message));
    }

    Ok(parsed.data)
}

pub fn resolve_github_token() -> Result<String> {
    resolve_github_token_with_source().map(|(token, _)| token)
}

pub fn resolve_github_token_with_source() -> Result<(String, GithubTokenSource)> {
    for env_name in GITHUB_TOKEN_ENV_VARS {
        if let Ok(token) = env::var(env_name) {
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Ok((token, GithubTokenSource::EnvironmentVariable(env_name)));
            }
        }
    }

    #[cfg(feature = "desktop")]
    {
        use std::process::Command;

        let output = Command::new("gh").args(["auth", "token"]).output()?;
        if !output.status.success() {
            return Err(anyhow!(
                "resolve github token from env or gh auth token failed"
            ));
        }

        let token = String::from_utf8(output.stdout)?.trim().to_string();
        if token.is_empty() {
            return Err(anyhow!("github token was empty"));
        }

        return Ok((token, GithubTokenSource::GithubCli));
    }

    #[allow(unreachable_code)]
    Err(anyhow!("github token not found in environment"))
}

fn encode_query_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_pr_graphql_response_allows_missing_created_at_in_checkrun_contexts() {
        let payload = r#"{
            "data": {
                "search": {
                    "nodes": [
                        {
                            "__typename": "PullRequest",
                            "title": "Example PR",
                            "url": "https://github.com/acme/repo/pull/1",
                            "number": 1,
                            "createdAt": "2026-05-07T20:00:33Z",
                            "updatedAt": "2026-05-08T10:44:59Z",
                            "reviewDecision": "CHANGES_REQUESTED",
                            "reviews": {
                                "nodes": [
                                    {
                                        "state": "COMMENTED",
                                        "submittedAt": "2026-05-08T10:44:59Z"
                                    }
                                ]
                            },
                            "repository": {
                                "name": "repo",
                                "owner": {
                                    "login": "acme"
                                }
                            },
                            "commits": {
                                "nodes": [
                                    {
                                        "commit": {
                                            "committedDate": "2026-05-07T20:00:16Z",
                                            "statusCheckRollup": {
                                                "state": "SUCCESS",
                                                "contexts": {
                                                    "nodes": [
                                                        {
                                                            "__typename": "CheckRun",
                                                            "status": "COMPLETED",
                                                            "conclusion": "SUCCESS",
                                                            "completedAt": "2026-05-07T20:00:56Z",
                                                            "startedAt": "2026-05-07T20:00:43Z"
                                                        }
                                                    ]
                                                }
                                            }
                                        }
                                    }
                                ]
                            }
                        }
                    ],
                    "pageInfo": {
                        "hasNextPage": false,
                        "endCursor": null
                    }
                }
            },
            "errors": []
        }"#;

        let parsed: GithubGraphQLResponse<GithubOpenPullRequestsResponseData> =
            serde_json::from_str(payload).expect("payload should deserialize");
        assert_eq!(parsed.data.search.nodes.len(), 1);
        assert_eq!(parsed.data.search.nodes[0].created_at.is_some(), true);
    }
}
