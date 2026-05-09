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
            size: Some(HeroSize::Medium),
            color: Some(Color::Primary),
            Container {
                Title {
                    size: TitleSize::H1,
                    spaced: true,
                    "GitHub review requests"
                }
                p { class: "subtitle", "Pull requests waiting on your review." }
                p {
                    class: "has-text-grey-lighter",
                    "Queries GitHub for review-requested:@me state:open."
                }
            }
        }

        Section {
            Container {
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
                        div { class: "level mb-5",
                            div { class: "level-left",
                                div { class: "level-item",
                                    Title { size: TitleSize::H3, "Open review requests" }
                                }
                            }
                            div { class: "level-right",
                                div { class: "level-item",
                                    span { class: "has-text-info has-text-weight-semibold is-size-7", "{prs.len()} open" }
                                }
                            }
                        }

                        for pr in prs {
                            PullRequestCard { pr: pr.clone() }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn PullRequestCard(pr: PullRequestSummary) -> Element {
    let (requested_relative, requested_exact) = format_requested_at(pr.requested_at);
    let (updated_relative, updated_exact) = format_updated_at(pr.updated_at);
    let repo = format!("{}/{}", pr.owner, pr.repo);
    let number = format!("#{}", pr.number);
    let url = pr.html_url.clone();
    let title = pr.title.clone();

    rsx! {
        div { class: "box mb-4 p-4", style: "border-left: 4px solid hsl(217, 71%, 53%);",
            a {
                href: url.clone(),
                target: "_blank",
                rel: "noreferrer noopener",
                class: "is-flex is-flex-wrap-wrap is-align-items-baseline gap-2 has-text-dark mb-2",
                span { class: "has-text-link has-text-weight-semibold is-size-7 is-uppercase", "{repo}" }
                span { class: "has-text-grey-light has-text-weight-light", "·" }
                span { class: "has-text-grey-dark has-text-weight-semibold is-family-monospace", "{number}" }
                span { class: "has-text-grey-light has-text-weight-light", "·" }
                span { class: "has-text-weight-semibold is-size-6", "{title}" }
            }

            p { class: "",
                span { class: "is-size-5 has-text-weight-bold", title: requested_exact, "Requested {requested_relative}" }
                " "
                span { class: "is-size-5 ", title: updated_exact, "Updated {updated_relative}" }
            }
        }
    }
}

fn format_requested_at(t: Option<OffsetDateTime>) -> (String, String) {
    match t {
        Some(t) => format_time_pair(t),
        None => ("unknown".to_string(), "Unknown requested time".to_string()),
    }
}

fn format_updated_at(t: OffsetDateTime) -> (String, String) {
    format_time_pair(t)
}

fn format_time_pair(t: OffsetDateTime) -> (String, String) {
    (format_relative_time(t), format_time(t))
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

fn format_time(t: OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc2822)
        .unwrap_or_else(|_| t.to_string())
}
