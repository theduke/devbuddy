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
        state
        updatedAt
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
        repository {
          name
          owner {
            login
          }
        }
        commits(last: 1) {
          nodes {
            commit {
              committedDate
              statusCheckRollup {
                state
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

pub const GITHUB_TOKEN_ENV_VARS: &[&str] = &[
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "GITHUB_API_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
];

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
    pub last_pushed_at: Option<OffsetDateTime>,
    pub review_decision: PullRequestReviewDecision,
    pub ci_status: PullRequestCiStatus,
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
    title: String,
    url: String,
    number: i64,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
    repository: GithubRepository,
    author: GithubPullRequestAuthor,
    timeline_items: GithubTimelineItemConn,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubOpenPullRequestsSearchNode {
    #[serde(rename = "__typename")]
    typename: String,
    title: String,
    url: String,
    number: i64,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
    review_decision: GithubReviewDecision,
    repository: GithubRepository,
    commits: GithubPullRequestCommitConn,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestAuthor {
    login: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubTimelineItemConn {
    nodes: Vec<GithubTimelineItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubTimelineItem {
    #[serde(rename = "__typename")]
    typename: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    requested_reviewer: GithubRequestedReviewer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRequestedReviewer {
    #[serde(rename = "__typename")]
    typename: String,
    login: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestCommitConn {
    nodes: Vec<GithubPullRequestCommitNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequestCommitNode {
    commit: GithubCommit,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubCommit {
    #[serde(with = "time::serde::rfc3339")]
    committed_date: OffsetDateTime,
    status_check_rollup: Option<GithubStatusCheckRollup>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubStatusCheckRollup {
    state: GithubStatusState,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GithubReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepository {
    name: String,
    owner: GithubRepositoryOwner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepositoryOwner {
    login: String,
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

                let requested_at =
                    requested_at_for_viewer(&node.timeline_items.nodes, &page.viewer.login);
                results.push(PullRequestSummary {
                    owner: node.repository.owner.login,
                    repo: node.repository.name,
                    number: node.number,
                    title: node.title,
                    author: node.author.login,
                    html_url: node.url,
                    updated_at: node.updated_at,
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

                let last_pushed_at = node
                    .commits
                    .nodes
                    .first()
                    .map(|node| node.commit.committed_date);
                let ci_status = node
                    .commits
                    .nodes
                    .first()
                    .and_then(|node| node.commit.status_check_rollup.as_ref())
                    .map(|rollup| rollup.state.into())
                    .unwrap_or(PullRequestCiStatus::Unknown);

                results.push(OpenPullRequestSummary {
                    owner: node.repository.owner.login,
                    repo: node.repository.name,
                    number: node.number,
                    title: node.title,
                    html_url: node.url,
                    opened_at: node.created_at,
                    last_pushed_at,
                    review_decision: node.review_decision.into(),
                    ci_status,
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
        if request.requested_reviewer.typename != "User"
            || request.requested_reviewer.login.as_deref() != Some(viewer_login)
        {
            continue;
        }

        newest = Some(match newest {
            Some(current) if current >= request.created_at => current,
            _ => request.created_at,
        });
    }

    if newest.is_some() {
        return newest;
    }

    requests.first().map(|request| request.created_at)
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
    for env_name in GITHUB_TOKEN_ENV_VARS {
        if let Ok(token) = env::var(env_name) {
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Ok(token);
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

        return Ok(token);
    }

    #[allow(unreachable_code)]
    Err(anyhow!("github token not found in environment"))
}
