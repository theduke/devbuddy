use crate::{
    source::github::{GithubClient, OpenPullRequestSummary, PullRequestSummary},
    store::types::{
        GithubReviewRequestItem, GithubUserPullRequestItem, Item, ItemKind, PullRequestCiStatus,
        PullRequestReviewDecision,
    },
    store::{FsStore, Store},
};
use dioxus::prelude::*;
use dioxus_bulma::{Color, Container, Hero, HeroSize, Notification, Section, Title, TitleSize};
use futures::{join, StreamExt};
use std::sync::Arc;
use std::{cmp::Ordering, time::Duration};
use time::OffsetDateTime;

#[derive(Clone, Copy)]
enum HomeCommand {
    Refresh,
    Sort(HomeSort),
    Grouping(HomeGrouping),
}

const AUTO_REFRESH_INTERVAL_SECS: u64 = 90;

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum HomeGrouping {
    Grouped,
    Flat,
}

impl HomeGrouping {
    fn label(self) -> &'static str {
        match self {
            HomeGrouping::Grouped => "grouped",
            HomeGrouping::Flat => "flat",
        }
    }
}

#[component]
pub fn Home() -> Element {
    let review_requests_loading = use_signal(|| true);
    let review_requests_error = use_signal(|| None::<String>);
    let review_requests_data = use_signal(|| None::<Vec<Item>>);
    let open_pull_requests_loading = use_signal(|| true);
    let open_pull_requests_error = use_signal(|| None::<String>);
    let open_pull_requests_data = use_signal(|| None::<Vec<Item>>);
    let sort_order = use_signal(|| HomeSort::Newest);
    let grouping = use_signal(|| HomeGrouping::Grouped);
    let store: Arc<dyn Store> = Arc::new(FsStore::new(None));

    let refresh = use_coroutine(move |mut rx| {
        let store = Arc::clone(&store);
        let review_requests_loading = review_requests_loading;
        let review_requests_error = review_requests_error;
        let review_requests_data = review_requests_data;
        let open_pull_requests_loading = open_pull_requests_loading;
        let open_pull_requests_error = open_pull_requests_error;
        let open_pull_requests_data = open_pull_requests_data;
        let mut sort_order = sort_order;
        let mut grouping = grouping;

        async move {
            let mut stored_items = load_stored_home_data(
                store.as_ref(),
                review_requests_loading,
                review_requests_error,
                review_requests_data,
                open_pull_requests_loading,
                open_pull_requests_error,
                open_pull_requests_data,
                sort_order(),
            )
            .await;

            refresh_home_data(
                store.as_ref(),
                &mut stored_items,
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
                        refresh_home_data(
                            store.as_ref(),
                            &mut stored_items,
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
                    HomeCommand::Grouping(mode) => {
                        grouping.set(mode);
                    }
                }
            }
        }
    });

    let _auto_refresh = use_future(move || {
        let refresh = refresh;
        async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(AUTO_REFRESH_INTERVAL_SECS));
            interval.tick().await;
            loop {
                interval.tick().await;
                refresh.send(HomeCommand::Refresh);
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
    let grouping_value = grouping();
    let is_loading = review_requests_loading_value || open_pull_requests_loading_value;
    let current_data_age_label = current_data_age_label(
        review_requests_data_value.as_ref(),
        open_pull_requests_data_value.as_ref(),
    );

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
                            div { class: "buttons has-addons review-sort-selector mr-3",
                                button { class: "button is-static review-sort-selector-label",
                                    span { class: "icon is-small",
                                        img {
                                            src: asset!("/assets/sort.svg"),
                                            alt: "",
                                            width: "16",
                                            height: "16",
                                        }
                                    }
                                    span { class: "has-text-weight-semibold has-text-grey-dark", "sort:" }
                                }
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
                            div { class: "buttons has-addons review-grouping-selector mr-3",
                                button { class: "button is-static review-grouping-selector-label",
                                    span { class: "has-text-weight-semibold has-text-grey-dark", "group:" }
                                }
                                button {
                                    class: if grouping_value == HomeGrouping::Grouped {
                                        "button is-info is-selected"
                                    } else {
                                        "button is-info is-light"
                                    },
                                    onclick: move |_| {
                                        refresh.send(HomeCommand::Grouping(HomeGrouping::Grouped))
                                    },
                                    span { "{HomeGrouping::Grouped.label()}" }
                                }
                                button {
                                    class: if grouping_value == HomeGrouping::Flat {
                                        "button is-success is-selected"
                                    } else {
                                        "button is-success is-light"
                                    },
                                    onclick: move |_| {
                                        refresh.send(HomeCommand::Grouping(HomeGrouping::Flat))
                                    },
                                    span { "{HomeGrouping::Flat.label()}" }
                                }
                            }
                            button {
                                class: if is_loading {
                                    "button is-loading is-inline-flex is-align-items-center is-justify-content-center"
                                } else {
                                    "button is-inline-flex is-align-items-center is-justify-content-center"
                                },
                                disabled: is_loading,
                                "aria-label": "Refresh",
                                onclick: move |_| refresh.send(HomeCommand::Refresh),
                                if !is_loading {
                                    span { class: "icon is-small m-0 is-flex is-align-items-center is-justify-content-center",
                                        img {
                                            src: asset!("/assets/refresh.svg"),
                                            alt: "",
                                            width: "16",
                                            height: "16",
                                        }
                                    }
                                }
                                span { class: "is-sr-only", "Refresh" }
                            }
                            if let Some(current_data_age_label) = current_data_age_label {
                                span { class: "ml-3 is-size-7 has-text-grey",
                                    "{current_data_age_label}"
                                }
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
                match grouping_value {
                    HomeGrouping::Grouped => rsx! {
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
                                                ReviewRequestCard { item: pr.clone() }
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
                                                OpenPullRequestCard { item: pr.clone() }
                                            }
                                        }
                                    },
                                    _ => rsx! { div {} },
                                }
                            }
                        }
                    },
                    HomeGrouping::Flat => rsx! {
                        div { class: "review-dashboard review-dashboard-flat",
                            FlatPullRequestList {
                                review_requests_loading: review_requests_loading_value,
                                review_requests_error: review_requests_error_value,
                                review_requests_data: review_requests_data_value,
                                open_pull_requests_loading: open_pull_requests_loading_value,
                                open_pull_requests_error: open_pull_requests_error_value,
                                open_pull_requests_data: open_pull_requests_data_value,
                                sort_order: sort_order_value,
                            }
                        }
                    },
                }
            }
        }
    }
}

async fn load_stored_home_data(
    store: &dyn Store,
    mut review_requests_loading: Signal<bool>,
    mut review_requests_error: Signal<Option<String>>,
    review_requests_data: Signal<Option<Vec<Item>>>,
    mut open_pull_requests_loading: Signal<bool>,
    mut open_pull_requests_error: Signal<Option<String>>,
    open_pull_requests_data: Signal<Option<Vec<Item>>>,
    sort_order: HomeSort,
) -> Vec<Item> {
    *review_requests_loading.write() = true;
    *review_requests_error.write() = None;
    *open_pull_requests_loading.write() = true;
    *open_pull_requests_error.write() = None;

    let stored_items = match store.load_items().await {
        Ok(items) => items,
        Err(err) => {
            eprintln!("failed to load stored home items: {err:#}");
            Vec::new()
        }
    };

    let mut stored_home_data = HomeData::from_items(stored_items.clone());
    stored_home_data.sort(sort_order);
    apply_home_data(
        stored_home_data,
        review_requests_loading,
        review_requests_data,
        open_pull_requests_loading,
        open_pull_requests_data,
    );

    stored_items
}

async fn refresh_home_data(
    store: &dyn Store,
    stored_items: &mut Vec<Item>,
    mut review_requests_loading: Signal<bool>,
    mut review_requests_error: Signal<Option<String>>,
    review_requests_data: Signal<Option<Vec<Item>>>,
    mut open_pull_requests_loading: Signal<bool>,
    mut open_pull_requests_error: Signal<Option<String>>,
    open_pull_requests_data: Signal<Option<Vec<Item>>>,
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

    let mut review_requests_data_value = Vec::new();
    let review_requests_loaded = match review_result {
        Ok(review_requests) => {
            let mut review_requests = map_review_requests(review_requests);
            sort_review_requests(&mut review_requests, sort_order);
            review_requests_data_value = review_requests;
            true
        }
        Err(err) => {
            *review_requests_error.write() = Some(err.to_string());
            false
        }
    };

    let mut open_pull_requests_data_value = Vec::new();
    let open_pull_requests_loaded = match open_result {
        Ok(open_pull_requests) => {
            let mut open_pull_requests = map_open_pull_requests(open_pull_requests);
            sort_open_pull_requests(&mut open_pull_requests, sort_order);
            open_pull_requests_data_value = open_pull_requests;
            true
        }
        Err(err) => {
            *open_pull_requests_error.write() = Some(err.to_string());
            false
        }
    };

    if review_requests_loaded && open_pull_requests_loaded {
        let fresh_items = combine_home_items_for_storage(
            review_requests_data_value.clone(),
            open_pull_requests_data_value.clone(),
        );
        if fresh_items != *stored_items {
            if let Err(err) = store.store_items(fresh_items.clone()).await {
                eprintln!("failed to persist refreshed home items: {err:#}");
            } else {
                *stored_items = fresh_items;
            }
        }
    }

    apply_home_data(
        HomeData {
            review_requests: review_requests_data_value,
            open_pull_requests: open_pull_requests_data_value,
        },
        review_requests_loading,
        review_requests_data,
        open_pull_requests_loading,
        open_pull_requests_data,
    );
}

fn apply_home_data(
    home_data: HomeData,
    mut review_requests_loading: Signal<bool>,
    mut review_requests_data: Signal<Option<Vec<Item>>>,
    mut open_pull_requests_loading: Signal<bool>,
    mut open_pull_requests_data: Signal<Option<Vec<Item>>>,
) {
    *review_requests_data.write() = Some(home_data.review_requests);
    *review_requests_loading.write() = false;
    *open_pull_requests_data.write() = Some(home_data.open_pull_requests);
    *open_pull_requests_loading.write() = false;
}

#[derive(Default)]
struct HomeData {
    review_requests: Vec<Item>,
    open_pull_requests: Vec<Item>,
}

impl HomeData {
    fn from_items(items: Vec<Item>) -> Self {
        let mut home_data = HomeData::default();
        for item in items {
            match &item.kind {
                ItemKind::GithubReviewRequest(_) => {
                    home_data.review_requests.push(item);
                }
                ItemKind::GithubUserPullRequest(_) => {
                    home_data.open_pull_requests.push(item);
                }
            }
        }
        home_data
    }

    fn sort(&mut self, sort_order: HomeSort) {
        sort_review_requests(&mut self.review_requests, sort_order);
        sort_open_pull_requests(&mut self.open_pull_requests, sort_order);
    }
}

fn combine_home_items_for_storage(
    mut review_requests: Vec<Item>,
    mut open_pull_requests: Vec<Item>,
) -> Vec<Item> {
    let mut items = Vec::with_capacity(review_requests.len() + open_pull_requests.len());
    items.append(&mut review_requests);
    items.append(&mut open_pull_requests);
    items
}

fn current_data_age_label(
    review_requests_data: Option<&Vec<Item>>,
    open_pull_requests_data: Option<&Vec<Item>>,
) -> Option<String> {
    latest_current_data_retrieved_at(review_requests_data, open_pull_requests_data)
        .map(|retrieved_at| format!("Updated {}", format_relative_time(retrieved_at)))
}

fn latest_current_data_retrieved_at(
    review_requests_data: Option<&Vec<Item>>,
    open_pull_requests_data: Option<&Vec<Item>>,
) -> Option<OffsetDateTime> {
    review_requests_data
        .into_iter()
        .chain(open_pull_requests_data)
        .flat_map(|items| items.iter().map(|item| item.retrieved_at))
        .max_by_key(|retrieved_at| retrieved_at.unix_timestamp())
}

fn sort_loaded_home_data(
    mut review_requests_data: Signal<Option<Vec<Item>>>,
    mut open_pull_requests_data: Signal<Option<Vec<Item>>>,
    sort_order: HomeSort,
) {
    if let Some(review_requests) = review_requests_data.write().as_mut() {
        sort_review_requests(review_requests, sort_order);
    }

    if let Some(open_pull_requests) = open_pull_requests_data.write().as_mut() {
        sort_open_pull_requests(open_pull_requests, sort_order);
    }
}

fn sort_review_requests(prs: &mut Vec<Item>, sort_order: HomeSort) {
    prs.sort_by(|a, b| match sort_order {
        HomeSort::Oldest => match (review_request_sort_time(a), review_request_sort_time(b)) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => review_request_fallback_time(a).cmp(&review_request_fallback_time(b)),
        },
        HomeSort::Newest => match (review_request_sort_time(a), review_request_sort_time(b)) {
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => review_request_fallback_time(b).cmp(&review_request_fallback_time(a)),
        },
    });
}

fn sort_open_pull_requests(prs: &mut Vec<Item>, sort_order: HomeSort) {
    prs.sort_by(|a, b| match sort_order {
        HomeSort::Oldest => match (
            open_pull_request_sort_time(a),
            open_pull_request_sort_time(b),
        ) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                open_pull_request_fallback_time(a).cmp(&open_pull_request_fallback_time(b))
            }
        },
        HomeSort::Newest => match (
            open_pull_request_sort_time(a),
            open_pull_request_sort_time(b),
        ) {
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                open_pull_request_fallback_time(b).cmp(&open_pull_request_fallback_time(a))
            }
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
fn FlatPullRequestList(
    review_requests_loading: bool,
    review_requests_error: Option<String>,
    review_requests_data: Option<Vec<Item>>,
    open_pull_requests_loading: bool,
    open_pull_requests_error: Option<String>,
    open_pull_requests_data: Option<Vec<Item>>,
    sort_order: HomeSort,
) -> Element {
    let review_requests = review_requests_data.unwrap_or_default();
    let open_pull_requests = open_pull_requests_data.unwrap_or_default();
    let items = combine_home_feed_items(review_requests, open_pull_requests, sort_order);
    let has_errors = review_requests_error.is_some() || open_pull_requests_error.is_some();
    let count_label = if has_errors {
        None
    } else {
        Some(format!("{} open", items.len()))
    };
    let review_requests_error = review_requests_error.as_ref();
    let open_pull_requests_error = open_pull_requests_error.as_ref();
    let is_initial_loading =
        items.is_empty() && (review_requests_loading || open_pull_requests_loading) && !has_errors;

    rsx! {
        div { class: "review-panel",
            PullRequestListHeader {
                title: "All Pull Requests".to_string(),
                subtitle: "Review requests and your open pull requests combined.".to_string(),
                count_label,
            }
            if let Some(err) = review_requests_error {
                Notification {
                    color: Some(Color::Danger),
                    p { "Review requests error: {err}" }
                }
            }
            if let Some(err) = open_pull_requests_error {
                Notification {
                    color: Some(Color::Danger),
                    p { "Open pull requests error: {err}" }
                }
            }
            if is_initial_loading {
                Notification {
                    color: Some(Color::Info),
                    "Loading pull requests…"
                }
            } else if items.is_empty() && !has_errors {
                Notification {
                    color: Some(Color::Success),
                    "No pull requests found."
                }
            } else {
                div { class: "review-card-stack",
                    for item in items {
                        HomeFeedCard { item }
                    }
                }
            }
        }
    }
}

#[component]
fn HomeFeedCard(item: Item) -> Element {
    match &item.kind {
        ItemKind::GithubReviewRequest(_) => rsx! {
            ReviewRequestCard { item: item.clone() }
        },
        ItemKind::GithubUserPullRequest(_) => rsx! {
            OpenPullRequestCard { item: item.clone() }
        },
    }
}

fn combine_home_feed_items(
    review_requests: Vec<Item>,
    open_pull_requests: Vec<Item>,
    sort_order: HomeSort,
) -> Vec<Item> {
    let mut items = Vec::with_capacity(review_requests.len() + open_pull_requests.len());
    items.extend(review_requests);
    items.extend(open_pull_requests);
    items.sort_by(|a, b| compare_home_feed_items(a, b, sort_order));
    items
}

fn compare_home_feed_items(a: &Item, b: &Item, sort_order: HomeSort) -> Ordering {
    match sort_order {
        HomeSort::Oldest => match (feed_item_sort_time(a), feed_item_sort_time(b)) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => feed_item_fallback_time(a).cmp(&feed_item_fallback_time(b)),
        },
        HomeSort::Newest => match (feed_item_sort_time(a), feed_item_sort_time(b)) {
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => feed_item_fallback_time(b).cmp(&feed_item_fallback_time(a)),
        },
    }
}

fn feed_item_sort_time(item: &Item) -> Option<OffsetDateTime> {
    match item {
        Item {
            kind: ItemKind::GithubReviewRequest(pr),
            ..
        } => pr.requested_at,
        Item {
            kind: ItemKind::GithubUserPullRequest(pr),
            ..
        } => pr.last_pushed_at,
    }
}

fn feed_item_fallback_time(item: &Item) -> OffsetDateTime {
    match item {
        Item {
            kind: ItemKind::GithubReviewRequest(pr),
            ..
        } => pr.updated_at,
        Item {
            kind: ItemKind::GithubUserPullRequest(pr),
            ..
        } => pr.opened_at,
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

fn map_review_requests(review_requests: Vec<PullRequestSummary>) -> Vec<Item> {
    let retrieved_at = OffsetDateTime::now_utc();
    review_requests
        .into_iter()
        .map(|pr| Item {
            kind: ItemKind::GithubReviewRequest(GithubReviewRequestItem {
                owner: pr.owner,
                repo: pr.repo,
                number: pr.number,
                title: pr.title,
                author: pr.author,
                html_url: pr.html_url,
                opened_at: pr.opened_at,
                last_pushed_at: pr.last_pushed_at,
                updated_at: pr.updated_at,
                requested_at: pr.requested_at,
            }),
            retrieved_at,
            ignore: false,
            ignore_until: None,
        })
        .collect()
}

fn map_open_pull_requests(open_pull_requests: Vec<OpenPullRequestSummary>) -> Vec<Item> {
    let retrieved_at = OffsetDateTime::now_utc();
    open_pull_requests
        .into_iter()
        .map(|pr| Item {
            kind: ItemKind::GithubUserPullRequest(GithubUserPullRequestItem {
                owner: pr.owner,
                repo: pr.repo,
                number: pr.number,
                title: pr.title,
                html_url: pr.html_url,
                opened_at: pr.opened_at,
                last_pushed_at: pr.last_pushed_at,
                review_decision: match pr.review_decision {
                    crate::source::github::PullRequestReviewDecision::Approved => {
                        PullRequestReviewDecision::Approved
                    }
                    crate::source::github::PullRequestReviewDecision::ChangesRequested => {
                        PullRequestReviewDecision::ChangesRequested
                    }
                    crate::source::github::PullRequestReviewDecision::ReviewRequired => {
                        PullRequestReviewDecision::ReviewRequired
                    }
                },
                ci_status: match pr.ci_status {
                    crate::source::github::PullRequestCiStatus::Failed => {
                        PullRequestCiStatus::Failed
                    }
                    crate::source::github::PullRequestCiStatus::InProgress => {
                        PullRequestCiStatus::InProgress
                    }
                    crate::source::github::PullRequestCiStatus::Success => {
                        PullRequestCiStatus::Success
                    }
                    crate::source::github::PullRequestCiStatus::Unknown => {
                        PullRequestCiStatus::Unknown
                    }
                },
                last_review_comment_at: pr.last_review_comment_at,
                last_changes_requested_at: pr.last_changes_requested_at,
                last_approved_at: pr.last_approved_at,
                last_ci_failure_at: pr.last_ci_failure_at,
                last_ci_success_at: pr.last_ci_success_at,
                last_ci_started_at: pr.last_ci_started_at,
            }),
            retrieved_at,
            ignore: false,
            ignore_until: None,
        })
        .collect()
}

fn review_request_sort_time(item: &Item) -> Option<OffsetDateTime> {
    match &item.kind {
        ItemKind::GithubReviewRequest(pr) => pr.requested_at,
        ItemKind::GithubUserPullRequest(_) => None,
    }
}

fn review_request_fallback_time(item: &Item) -> OffsetDateTime {
    match &item.kind {
        ItemKind::GithubReviewRequest(pr) => pr.updated_at,
        ItemKind::GithubUserPullRequest(_) => item.retrieved_at,
    }
}

fn open_pull_request_sort_time(item: &Item) -> Option<OffsetDateTime> {
    match &item.kind {
        ItemKind::GithubReviewRequest(_) => None,
        ItemKind::GithubUserPullRequest(pr) => pr.last_pushed_at,
    }
}

fn open_pull_request_fallback_time(item: &Item) -> OffsetDateTime {
    match &item.kind {
        ItemKind::GithubReviewRequest(_) => item.retrieved_at,
        ItemKind::GithubUserPullRequest(pr) => pr.opened_at,
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
