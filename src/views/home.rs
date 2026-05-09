use crate::source::github::{
    GithubClient, OpenPullRequestSummary, PullRequestCiStatus, PullRequestReviewDecision,
    PullRequestSummary,
};
use dioxus::prelude::*;
use dioxus_bulma::{Color, Container, Hero, HeroSize, Notification, Section, Title, TitleSize};
use time::OffsetDateTime;

#[component]
pub fn Home() -> Element {
    let review_requests =
        use_resource::<Result<Vec<PullRequestSummary>, anyhow::Error>, _>(move || async move {
            let client = GithubClient::new()?;
            client.pull_requests_requested_for_review().await
        });
    let open_pull_requests =
        use_resource::<Result<Vec<OpenPullRequestSummary>, anyhow::Error>, _>(move || async move {
            let client = GithubClient::new()?;
            client.open_pull_requests_for_user().await
        });

    rsx! {
        Hero {
            size: Some(HeroSize::Small),
            class: "review-hero",
            Container {
                class: "review-container",
                div { class: "is-flex is-align-items-center",
                    img {
                        class: "review-hero-icon mr-4",
                        src: asset!("/assets/github-mark.svg"),
                        alt: "GitHub",
                        width: "48",
                        height: "48",
                    }
                    div {
                        Title {
                            size: TitleSize::H1,
                            class: "mb-2 review-hero-title",
                            "GitHub pull requests"
                        }
                        p { class: "subtitle is-6 has-text-grey mb-0",
                            "Review requests and your open pull requests on GitHub."
                        }
                    }
                }
            }
        }

        Section {
            class: "review-section pt-5",
            Container {
                class: "review-container",
                div { class: "review-dashboard",
                    div { class: "review-panel",
                        PullRequestListHeader {
                            title: "Open review requests".to_string(),
                            subtitle: "Pull requests currently waiting on your review.".to_string(),
                            count_label: match &*review_requests.read() {
                                Some(Ok(prs)) => Some(format!("{} open", prs.len())),
                                _ => None,
                            },
                        }
                        match &*review_requests.read() {
                            None => rsx! {
                                Notification {
                                    color: Some(Color::Info),
                                    "Loading review requests…"
                                }
                            },
                            Some(Err(err)) => rsx! {
                                Notification {
                                    color: Some(Color::Danger),
                                    p { "Error: {err}" }
                                }
                            },
                            Some(Ok(prs)) if prs.is_empty() => rsx! {
                                Notification {
                                    color: Some(Color::Success),
                                    "No active review requests."
                                }
                            },
                            Some(Ok(prs)) => rsx! {
                                div { class: "review-card-stack",
                                    for pr in prs {
                                        ReviewRequestCard { pr: pr.clone() }
                                    }
                                }
                            },
                        }
                    }

                    div { class: "review-panel",
                        PullRequestListHeader {
                            title: "Your open pull requests".to_string(),
                            subtitle: "Authored pull requests with the next action derived from review and CI state.".to_string(),
                            count_label: match &*open_pull_requests.read() {
                                Some(Ok(prs)) => Some(format!("{} open", prs.len())),
                                _ => None,
                            },
                        }
                        match &*open_pull_requests.read() {
                            None => rsx! {
                                Notification {
                                    color: Some(Color::Info),
                                    "Loading your pull requests…"
                                }
                            },
                            Some(Err(err)) => rsx! {
                                Notification {
                                    color: Some(Color::Danger),
                                    p { "Error: {err}" }
                                }
                            },
                            Some(Ok(prs)) if prs.is_empty() => rsx! {
                                Notification {
                                    color: Some(Color::Success),
                                    "No open authored pull requests."
                                }
                            },
                            Some(Ok(prs)) => rsx! {
                                div { class: "review-card-stack",
                                    for pr in prs {
                                        OpenPullRequestCard { pr: pr.clone() }
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PullRequestListHeader(title: String, subtitle: String, count_label: Option<String>) -> Element {
    rsx! {
        div { class: "level is-mobile mb-4 review-list-header",
            div { class: "level-left",
                div { class: "level-item",
                    div {
                        h2 { class: "title is-5 has-text-grey-dark mb-1", "{title}" }
                        p { class: "review-list-subtitle mb-0", "{subtitle}" }
                    }
                }
            }
            if let Some(count_label) = count_label {
                div { class: "level-right",
                    div { class: "level-item",
                        span { class: "tag is-info is-light is-medium has-text-weight-semibold",
                            "{count_label}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ReviewRequestCard(pr: PullRequestSummary) -> Element {
    let repo = format!("{}/{}", pr.owner, pr.repo);
    let number = format!("#{}", pr.number);

    rsx! {
        PullRequestBox {
            url: pr.html_url,
            title: pr.title,
            repo,
            number,
            meta_suffix: Some(format!("@{}", pr.author)),
            subtitle: None,
            status_label: "NEEDS REVIEW",
            age_label: "Requested",
            age_value: format_requested_at(pr.requested_at),
            age_tone: age_tone_suffix(pr.requested_at),
            icon_kind: PullRequestIconKind::ReviewRequest,
        }
    }
}

#[component]
fn OpenPullRequestCard(pr: OpenPullRequestSummary) -> Element {
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
            meta_suffix: None,
            subtitle: Some(subtitle),
            status_label: action.label,
            age_label: action.age_label,
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
    age_label: &'static str,
    age_value: String,
    age_tone: &'static str,
    icon_kind: PullRequestIconKind,
) -> Element {
    let age_class = format!("review-age review-age-{age_tone} has-text-weight-bold");
    let box_tone_class = format!("review-pr-box-{age_tone}");
    let age_section_class = format!("review-pr-age-section review-pr-age-section-{age_tone}");
    let status_class = format!("review-pr-status review-pr-status-{age_tone}");
    let status_label_class = format!("review-pr-status-label review-pr-status-label-{age_tone}");
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
                div { class: "{status_class}",
                    span { class: "{status_label_class}", "{status_label}" }
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
                div { class: "{age_section_class}",
                    span { class: "review-pr-age-label-top", "{age_label}" }
                    span { class: "{age_class}", "{age_value}" }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct OpenPullRequestAction {
    label: &'static str,
    age_label: &'static str,
    at: Option<OffsetDateTime>,
}

fn derive_open_pull_request_action(pr: &OpenPullRequestSummary) -> OpenPullRequestAction {
    if pr.ci_status == PullRequestCiStatus::Failed {
        return OpenPullRequestAction {
            label: "CI FAIL",
            age_label: "CI failed",
            at: pr
                .last_ci_failure_at
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    if pr.review_decision == PullRequestReviewDecision::ChangesRequested {
        return OpenPullRequestAction {
            label: "CHANGES REQUESTED",
            age_label: "Requested",
            at: pr
                .last_changes_requested_at
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    if pr.last_review_comment_at.is_some() {
        return OpenPullRequestAction {
            label: "REVIEW COMMENTS",
            age_label: "Commented",
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
            age_label: "Ready since",
            at: latest_timestamp(pr.last_approved_at, pr.last_ci_success_at)
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    if pr.ci_status == PullRequestCiStatus::InProgress {
        return OpenPullRequestAction {
            label: "CI RUNNING",
            age_label: "CI started",
            at: pr
                .last_ci_started_at
                .or(pr.last_pushed_at)
                .or(Some(pr.opened_at)),
        };
    }

    OpenPullRequestAction {
        label: "NEEDS REVIEW",
        age_label: "Last pushed",
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

fn format_relative_time(t: OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let diff = now.unix_timestamp() - t.unix_timestamp();
    let (value, unit, suffix) = if diff.abs() < 60 {
        return "just now".to_string();
    } else if diff.abs() < 3_600 {
        (
            diff.abs() / 60,
            "minute",
            if diff >= 0 { "ago" } else { "from now" },
        )
    } else if diff.abs() < 86_400 {
        (
            diff.abs() / 3_600,
            "hour",
            if diff >= 0 { "ago" } else { "from now" },
        )
    } else if diff.abs() < 604_800 {
        (
            diff.abs() / 86_400,
            "day",
            if diff >= 0 { "ago" } else { "from now" },
        )
    } else if diff.abs() < 31_536_000 {
        (
            diff.abs() / 604_800,
            "week",
            if diff >= 0 { "ago" } else { "from now" },
        )
    } else {
        (
            diff.abs() / 31_536_000,
            "year",
            if diff >= 0 { "ago" } else { "from now" },
        )
    };

    let plural = if value == 1 { "" } else { "s" };
    format!("{} {}{} {}", value, unit, plural, suffix)
}
