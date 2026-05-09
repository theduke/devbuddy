use super::format_relative_time;
use crate::store::types::{
    GithubUserPullRequestItem, Item, ItemKind, PullRequestCiStatus, PullRequestReviewDecision,
};
use dioxus::prelude::*;
use time::OffsetDateTime;

#[component]
pub fn GithubItemCard(item: Item) -> Element {
    match &item.kind {
        ItemKind::GithubReviewRequest(_) => rsx! {
            ReviewRequestCard { item: item.clone() }
        },
        ItemKind::GithubUserPullRequest(_) => rsx! {
            OpenPullRequestCard { item: item.clone() }
        },
    }
}

#[component]
fn ReviewRequestCard(item: Item) -> Element {
    let Item {
        kind: ItemKind::GithubReviewRequest(pr),
        ..
    } = item
    else {
        unreachable!("review request card received non-review item");
    };
    let repo = format!("{}/{}", pr.owner, pr.repo);
    let number = format!("#{}", pr.number);
    let subtitle = format!(
        "Opened {} · Last pushed {}",
        format_relative_time(pr.opened_at),
        format_requested_at(pr.last_pushed_at)
    );

    rsx! {
        PullRequestBox {
            url: pr.html_url,
            title: pr.title,
            repo,
            number,
            meta_suffix: Some(format!("@{}", pr.author)),
            subtitle: Some(subtitle),
            status_label: "NEEDS REVIEW",
            age_value: format_requested_at(pr.requested_at),
            age_tone: age_tone_suffix(pr.requested_at),
            icon_kind: PullRequestIconKind::ReviewRequest,
        }
    }
}

#[component]
fn OpenPullRequestCard(item: Item) -> Element {
    let Item {
        kind: ItemKind::GithubUserPullRequest(pr),
        ..
    } = item
    else {
        unreachable!("open pull request card received non-open item");
    };
    let action = derive_open_pull_request_action(&pr);
    let repo = format!("{}/{}", pr.owner, pr.repo);
    let number = format!("#{}", pr.number);
    let subtitle = format!(
        "Opened {} · Last pushed {}",
        format_relative_time(pr.opened_at),
        format_requested_at(pr.last_pushed_at)
    );

    rsx! {
        PullRequestBox {
            url: pr.html_url,
            title: pr.title,
            repo,
            number,
            meta_suffix: Some("@me".to_string()),
            subtitle: Some(subtitle),
            status_label: action.label,
            age_value: format_requested_at(action.at),
            age_tone: age_tone_suffix(action.at),
            icon_kind: PullRequestIconKind::OwnPullRequest,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PullRequestIconKind {
    ReviewRequest,
    OwnPullRequest,
}

#[component]
fn PullRequestBox(
    url: String,
    title: String,
    repo: String,
    number: String,
    meta_suffix: Option<String>,
    subtitle: Option<String>,
    status_label: &'static str,
    age_value: String,
    age_tone: &'static str,
    icon_kind: PullRequestIconKind,
) -> Element {
    let age_class = format!("review-age review-age-{age_tone} has-text-weight-bold");
    let box_tone_class = format!("review-pr-box-{age_tone}");
    let action_section_class =
        format!("review-pr-action-section review-pr-action-section-{age_tone}");
    let action_label_class = format!("review-pr-action-label review-pr-action-label-{age_tone}");
    let icon_class = match icon_kind {
        PullRequestIconKind::ReviewRequest => "review-pr-icons review-pr-icons-review",
        PullRequestIconKind::OwnPullRequest => "review-pr-icons review-pr-icons-own",
    };

    rsx! {
        div { class: "box review-pr-box p-0 mb-0 {box_tone_class}",
            a {
                href: url,
                target: "_blank",
                rel: "noreferrer noopener",
                class: "review-pr-link has-text-dark",
                div { class: "{icon_class}",
                    img {
                        src: asset!("/assets/github-mark.svg"),
                        alt: "GitHub",
                        width: "22",
                        height: "22",
                    }
                    match icon_kind {
                        PullRequestIconKind::ReviewRequest => rsx! {
                            img {
                                src: asset!("/assets/pull-request.svg"),
                                alt: "Pull request waiting for review",
                                width: "22",
                                height: "22",
                            }
                        },
                        PullRequestIconKind::OwnPullRequest => rsx! {
                            img {
                                src: asset!("/assets/branch.svg"),
                                alt: "Your pull request",
                                width: "22",
                                height: "22",
                            }
                        },
                    }
                }
                div { class: "review-pr-content",
                    div { class: "review-pr-meta-row mb-1",
                        span { class: "review-repo has-text-weight-bold mr-2", "{repo}" }
                        span { class: "review-number is-family-monospace has-text-weight-semibold mr-2", "{number}" }
                        if let Some(meta_suffix) = meta_suffix {
                            span { class: "review-author has-text-weight-semibold", "{meta_suffix}" }
                        }
                    }
                    p { class: "review-pr-title has-text-weight-semibold", "{title}" }
                    if let Some(subtitle) = subtitle {
                        p { class: "review-pr-subtitle", "{subtitle}" }
                    }
                }
                div { class: "{action_section_class}",
                    span { class: "{action_label_class}", "{status_label}" }
                    span { class: "{age_class}", "{age_value}" }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct OpenPullRequestAction {
    label: &'static str,
    at: Option<OffsetDateTime>,
}

fn derive_open_pull_request_action(pr: &GithubUserPullRequestItem) -> OpenPullRequestAction {
    if pr.ci_status == PullRequestCiStatus::Failed {
        return OpenPullRequestAction {
            label: "CI FAIL",
            at: pr
                .last_ci_failure_at
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    if pr.review_decision == PullRequestReviewDecision::ChangesRequested {
        return OpenPullRequestAction {
            label: "CHANGES REQUESTED",
            at: pr
                .last_changes_requested_at
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    if pr.last_review_comment_at.is_some() {
        return OpenPullRequestAction {
            label: "REVIEW COMMENTS",
            at: pr
                .last_review_comment_at
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    if pr.review_decision == PullRequestReviewDecision::Approved
        && pr.ci_status == PullRequestCiStatus::Success
    {
        return OpenPullRequestAction {
            label: "READY",
            at: latest_timestamp(pr.last_approved_at, pr.last_ci_success_at)
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    if pr.ci_status == PullRequestCiStatus::InProgress {
        return OpenPullRequestAction {
            label: "CI RUNNING",
            at: pr
                .last_ci_started_at
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    OpenPullRequestAction {
        label: "NEEDS REVIEW",
        at: pr.last_pushed_at.or(Some(pr.opened_at)),
    }
}

fn latest_timestamp(
    a: Option<OffsetDateTime>,
    b: Option<OffsetDateTime>,
) -> Option<OffsetDateTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (None, None) => None,
    }
}

fn age_tone_suffix(t: Option<OffsetDateTime>) -> &'static str {
    let Some(t) = t else {
        return "unknown";
    };
    let age_seconds = (OffsetDateTime::now_utc().unix_timestamp() - t.unix_timestamp()).max(0);
    if age_seconds <= 4 * 3_600 {
        "green"
    } else if age_seconds <= 2 * 86_400 {
        "yellow"
    } else if age_seconds <= 5 * 86_400 {
        "orange"
    } else {
        "red"
    }
}

fn format_requested_at(t: Option<OffsetDateTime>) -> String {
    match t {
        Some(t) => format_relative_time(t),
        None => "unknown".to_string(),
    }
}
