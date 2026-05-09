use crate::source::github::{GithubClient, PullRequestSummary};
use dioxus::prelude::*;
use dioxus_bulma::{Color, Container, Hero, HeroSize, Notification, Section, Title, TitleSize};
use time::OffsetDateTime;

#[component]
pub fn Home() -> Element {
    let requests =
        use_resource::<Result<Vec<PullRequestSummary>, anyhow::Error>, _>(move || async move {
            let client = GithubClient::new()?;
            client.pull_requests_requested_for_review().await
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
                            "Review requests"
                        }
                        p { class: "subtitle is-6 has-text-grey mb-0",
                            "Pull requests waiting on your review on GitHub."
                        }
                    }
                }
            }
        }

        Section {
            class: "review-section pt-5",
            Container {
                class: "review-container",
                match &*requests.read() {
                    None => rsx! {
                        Notification {
                            color: Some(Color::Info),
                            "Loading…"
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
                        div { class: "level is-mobile mb-4 review-list-header",
                            div { class: "level-left",
                                div { class: "level-item",
                                    h2 { class: "title is-5 has-text-grey-dark mb-0",
                                        "Open review requests"
                                    }
                                }
                            }
                            div { class: "level-right",
                                div { class: "level-item",
                                    span { class: "tag is-info is-light is-medium has-text-weight-semibold",
                                        "{prs.len()} open"
                                    }
                                }
                            }
                        }

                        div { class: "review-card-stack",
                            for pr in prs {
                                PullRequestCard { pr: pr.clone() }
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn PullRequestCard(pr: PullRequestSummary) -> Element {
    let requested_relative = format_requested_at(pr.requested_at);
    let age_tone = age_tone_suffix(pr.requested_at);
    let requested_class = format!("review-age review-age-{age_tone} has-text-weight-bold");
    let box_tone_class = format!("review-pr-box-{age_tone}");
    let age_section_class = format!("review-pr-age-section review-pr-age-section-{age_tone}");
    let author = format!("@{}", pr.author);
    let repo = format!("{}/{}", pr.owner, pr.repo);
    let number = format!("#{}", pr.number);
    let url = pr.html_url.clone();
    let title = pr.title.clone();

    rsx! {
        div { class: "box review-pr-box p-0 mb-0 {box_tone_class}",
            a {
                href: url.clone(),
                target: "_blank",
                rel: "noreferrer noopener",
                class: "review-pr-link has-text-dark",
                div { class: "review-pr-icons",
                    img {
                        src: asset!("/assets/github-mark.svg"),
                        alt: "GitHub",
                        width: "22",
                        height: "22",
                    }
                    img {
                        src: asset!("/assets/pull-request.svg"),
                        alt: "Pull request",
                        width: "22",
                        height: "22",
                    }
                }
                div { class: "review-pr-status",
                    span { class: "review-pr-status-label", "NEEDS REVIEW" }
                }
                div { class: "review-pr-content",
                    div { class: "review-pr-meta-row mb-1",
                        span { class: "review-repo has-text-weight-bold mr-2", "{repo}" }
                        span { class: "review-number is-family-monospace has-text-weight-semibold mr-2", "{number}" }
                        span { class: "review-author has-text-weight-semibold", "{author}" }
                    }
                    p { class: "review-pr-title has-text-weight-semibold", "{title}" }
                }
                div { class: "{age_section_class}",
                    span { class: "review-pr-age-label-top", "Requested" }
                    span { class: "{requested_class}", "{requested_relative}" }
                }
            }
        }
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
