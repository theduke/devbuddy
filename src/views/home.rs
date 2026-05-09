use crate::source::github::{
    GithubClient, OpenPullRequestSummary, PullRequestCiStatus, PullRequestReviewDecision,
    PullRequestSummary,
};
use dioxus::prelude::*;
use dioxus_bulma::{Color, Container, Hero, HeroSize, Notification, Section, Title, TitleSize};
use futures::{join, StreamExt};
use time::OffsetDateTime;

#[derive(Clone, Copy)]
enum HomeCommand {
    Refresh,
    Sort(HomeSort),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HomeSort {
    Oldest,
    Newest,
}

impl HomeSort {
    fn label(self) -> &'static str {
        match self {
            HomeSort::Oldest => "oldest",
            HomeSort::Newest => "newest",
        }
    }
}

#[component]
pub fn Home() -> Element {
    let review_requests_loading = use_signal(|| true);
    let review_requests_error = use_signal(|| None::<String>);
    let review_requests_data = use_signal(|| None::<Vec<PullRequestSummary>>);
    let open_pull_requests_loading = use_signal(|| true);
    let open_pull_requests_error = use_signal(|| None::<String>);
    let open_pull_requests_data = use_signal(|| None::<Vec<OpenPullRequestSummary>>);
    let sort_order = use_signal(|| HomeSort::Newest);

    let refresh = use_coroutine(move |mut rx| {
        let review_requests_loading = review_requests_loading;
        let review_requests_error = review_requests_error;
        let review_requests_data = review_requests_data;
        let open_pull_requests_loading = open_pull_requests_loading;
        let open_pull_requests_error = open_pull_requests_error;
        let open_pull_requests_data = open_pull_requests_data;
        let mut sort_order = sort_order;

        async move {
            load_home_data(
                review_requests_loading,
                review_requests_error,
                review_requests_data,
                open_pull_requests_loading,
                open_pull_requests_error,
                open_pull_requests_data,
                sort_order(),
            )
            .await;

            while let Some(command) = rx.next().await {
                match command {
                    HomeCommand::Refresh => {
                        load_home_data(
                            review_requests_loading,
                            review_requests_error,
                            review_requests_data,
                            open_pull_requests_loading,
                            open_pull_requests_error,
                            open_pull_requests_data,
                            sort_order(),
                        )
                        .await;
                    }
                    HomeCommand::Sort(order) => {
                        sort_loaded_home_data(review_requests_data, open_pull_requests_data, order);
                        sort_order.set(order);
                    }
                }
            }
        }
    });

    let review_requests_loading_value = review_requests_loading();
    let review_requests_error_value = review_requests_error();
    let review_requests_data_value = review_requests_data();
    let open_pull_requests_loading_value = open_pull_requests_loading();
    let open_pull_requests_error_value = open_pull_requests_error();
    let open_pull_requests_data_value = open_pull_requests_data();
    let sort_order_value = sort_order();
    let is_loading = review_requests_loading_value || open_pull_requests_loading_value;

    rsx! {
        Hero {
            size: Some(HeroSize::Small),
            class: "review-hero",
            Container {
                class: "review-container",
                div { class: "is-flex is-justify-content-space-between is-align-items-center is-flex-wrap-wrap",
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
                    div {
                        class: "mt-4 mt-0-desktop",
                        div { class: "is-flex is-flex-wrap-wrap is-align-items-center is-justify-content-flex-end",
                            div { class: "is-flex is-align-items-center mr-3",
                                span { class: "icon is-small mr-1",
                                    img {
                                        src: asset!("/assets/sort.svg"),
                                        alt: "",
                                        width: "16",
                                        height: "16",
                                    }
                                }
                                span { class: "has-text-weight-semibold has-text-grey-dark", "sort:" }
                            }
                            div { class: "buttons has-addons mb-0 mr-3",
                                button {
                                    class: if sort_order_value == HomeSort::Oldest {
                                        "button is-warning is-selected"
                                    } else {
                                        "button is-warning is-light"
                                    },
                                    disabled: is_loading,
                                    onclick: move |_| refresh.send(HomeCommand::Sort(HomeSort::Oldest)),
                                    span { class: "icon is-small mr-1",
                                        img {
                                            src: asset!("/assets/up-arrow.svg"),
                                            alt: "",
                                            width: "16",
                                            height: "16",
                                        }
                                    }
                                    span { "{HomeSort::Oldest.label()}" }
                                }
                                button {
                                    class: if sort_order_value == HomeSort::Newest {
                                        "button is-info is-selected"
                                    } else {
                                        "button is-info is-light"
                                    },
                                    disabled: is_loading,
                                    onclick: move |_| refresh.send(HomeCommand::Sort(HomeSort::Newest)),
                                    span { class: "icon is-small mr-1",
                                        img {
                                            src: asset!("/assets/down-arrow.svg"),
                                            alt: "",
                                            width: "16",
                                            height: "16",
                                        }
                                    }
                                    span { "{HomeSort::Newest.label()}" }
                                }
                            }
                            button {
                                class: if is_loading {
                                    "button is-loading"
                                } else {
                                    "button"
                                },
                                disabled: is_loading,
                                onclick: move |_| refresh.send(HomeCommand::Refresh),
                                span { class: "icon is-small",
                                    img {
                                        src: asset!("/assets/refresh.svg"),
                                        alt: "",
                                        width: "16",
                                        height: "16",
                                    }
                                }
                                span { class: "is-sr-only", "Refresh" }
                            }
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
                            title: "Review Requests".to_string(),
                            subtitle: "...".to_string(),
                            count_label: if review_requests_error_value.is_none() {
                                review_requests_data_value
                                    .as_ref()
                                    .map(|prs| format!("{} open", prs.len()))
                            } else {
                                None
                            },
                        }
                        match (
                            review_requests_loading_value,
                            review_requests_error_value,
                            review_requests_data_value,
                        ) {
                            (_, Some(err), _) => rsx! {
                                Notification {
                                    color: Some(Color::Danger),
                                    p { "Error: {err}" }
                                }
                            },
                            (true, None, None) => rsx! {
                                Notification {
                                    color: Some(Color::Info),
                                    "Loading review requests…"
                                }
                            },
                            (_, None, Some(prs)) if prs.is_empty() => rsx! {
                                Notification {
                                    color: Some(Color::Success),
                                    "No active review requests."
                                }
                            },
                            (_, None, Some(prs)) => rsx! {
                                div { class: "review-card-stack",
                                    for pr in prs {
                                        ReviewRequestCard { pr: pr.clone() }
                                    }
                                }
                            },
                            _ => rsx! { div {} },
                        }
                    }

                    div { class: "review-panel",
                        PullRequestListHeader {
                            title: "My PRs".to_string(),
                            subtitle: "...",
                            count_label: if open_pull_requests_error_value.is_none() {
                                open_pull_requests_data_value
                                    .as_ref()
                                    .map(|prs| format!("{} open", prs.len()))
                            } else {
                                None
                            },
                        }
                        match (
                            open_pull_requests_loading_value,
                            open_pull_requests_error_value,
                            open_pull_requests_data_value,
                        ) {
                            (_, Some(err), _) => rsx! {
                                Notification {
                                    color: Some(Color::Danger),
                                    p { "Error: {err}" }
                                }
                            },
                            (true, None, None) => rsx! {
                                Notification {
                                    color: Some(Color::Info),
                                    "Loading your pull requests…"
                                }
                            },
                            (_, None, Some(prs)) if prs.is_empty() => rsx! {
                                Notification {
                                    color: Some(Color::Success),
                                    "No open authored pull requests."
                                }
                            },
                            (_, None, Some(prs)) => rsx! {
                                div { class: "review-card-stack",
                                    for pr in prs {
                                        OpenPullRequestCard { pr: pr.clone() }
                                    }
                                }
                            },
                            _ => rsx! { div {} },
                        }
                    }
                }
            }
        }
    }
}

async fn load_home_data(
    mut review_requests_loading: Signal<bool>,
    mut review_requests_error: Signal<Option<String>>,
    mut review_requests_data: Signal<Option<Vec<PullRequestSummary>>>,
    mut open_pull_requests_loading: Signal<bool>,
    mut open_pull_requests_error: Signal<Option<String>>,
    mut open_pull_requests_data: Signal<Option<Vec<OpenPullRequestSummary>>>,
    sort_order: HomeSort,
) {
    *review_requests_loading.write() = true;
    *review_requests_error.write() = None;
    *open_pull_requests_loading.write() = true;
    *open_pull_requests_error.write() = None;

    let client = match GithubClient::new() {
        Ok(client) => client,
        Err(err) => {
            let message = err.to_string();
            *review_requests_error.write() = Some(message.clone());
            *open_pull_requests_error.write() = Some(message);
            *review_requests_loading.write() = false;
            *open_pull_requests_loading.write() = false;
            return;
        }
    };

    let review_requests = client.pull_requests_requested_for_review();
    let open_pull_requests = client.open_pull_requests_for_user();
    let (review_result, open_result) = join!(review_requests, open_pull_requests);

    match review_result {
        Ok(mut review_requests) => {
            sort_review_requests(&mut review_requests, sort_order);
            *review_requests_data.write() = Some(review_requests);
        }
        Err(err) => {
            *review_requests_error.write() = Some(err.to_string());
        }
    }
    *review_requests_loading.write() = false;

    match open_result {
        Ok(mut open_pull_requests) => {
            sort_open_pull_requests(&mut open_pull_requests, sort_order);
            *open_pull_requests_data.write() = Some(open_pull_requests);
        }
        Err(err) => {
            *open_pull_requests_error.write() = Some(err.to_string());
        }
    }
    *open_pull_requests_loading.write() = false;
}

fn sort_loaded_home_data(
    mut review_requests_data: Signal<Option<Vec<PullRequestSummary>>>,
    mut open_pull_requests_data: Signal<Option<Vec<OpenPullRequestSummary>>>,
    sort_order: HomeSort,
) {
    if let Some(review_requests) = review_requests_data.write().as_mut() {
        sort_review_requests(review_requests, sort_order);
    }

    if let Some(open_pull_requests) = open_pull_requests_data.write().as_mut() {
        sort_open_pull_requests(open_pull_requests, sort_order);
    }
}

fn sort_review_requests(prs: &mut Vec<PullRequestSummary>, sort_order: HomeSort) {
    prs.sort_by(|a, b| match sort_order {
        HomeSort::Oldest => match (a.requested_at, b.requested_at) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.updated_at.cmp(&b.updated_at),
        },
        HomeSort::Newest => match (a.requested_at, b.requested_at) {
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.updated_at.cmp(&a.updated_at),
        },
    });
}

fn sort_open_pull_requests(prs: &mut Vec<OpenPullRequestSummary>, sort_order: HomeSort) {
    prs.sort_by(|a, b| match sort_order {
        HomeSort::Oldest => match (a.last_pushed_at, b.last_pushed_at) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.opened_at.cmp(&b.opened_at),
        },
        HomeSort::Newest => match (a.last_pushed_at, b.last_pushed_at) {
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.opened_at.cmp(&a.opened_at),
        },
    });
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

fn derive_open_pull_request_action(pr: &OpenPullRequestSummary) -> OpenPullRequestAction {
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
