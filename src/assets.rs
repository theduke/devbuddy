#[cfg(any(feature = "desktop", feature = "native"))]
mod _impl {
    use rust_embed::Embed;

    #[derive(Embed)]
    #[folder = "assets/"]
    pub struct EmbeddedAssets;

    pub const FAVICON: &str = "/embedded/favicon.ico";
    pub const MAIN_CSS: &str = "/embedded/styling/main.css";
    pub const BULMA_CSS: &str = "/embedded/bulma_v1.0.4.min.css";
    pub const HEADER: &str = "/embedded/header.svg";
    pub const GITHUB_MARK: &str = "/embedded/github-mark.svg";
    pub const PULL_REQUEST: &str = "/embedded/pull-request.svg";
    pub const BRANCH: &str = "/embedded/branch.svg";
    pub const SORT: &str = "/embedded/sort.svg";
    pub const UP_ARROW: &str = "/embedded/up-arrow.svg";
    pub const DOWN_ARROW: &str = "/embedded/down-arrow.svg";
    pub const REFRESH: &str = "/embedded/refresh.svg";
    pub const GEAR: &str = "/embedded/gear.svg";

    pub fn install() {
        #[cfg(feature = "desktop")]
        {
            use dioxus::desktop::{use_asset_handler, wry::http::Response};

            use_asset_handler("embedded", |request, responder| {
                let path = request.uri().path();
                let Some(relative_path) = path.strip_prefix("/embedded/") else {
                    return;
                };

                let Some(file) = EmbeddedAssets::get(relative_path) else {
                    responder.respond(Response::builder().status(404).body(Vec::new()).unwrap());
                    return;
                };

                let mime = match relative_path.rsplit('.').next() {
                    Some("css") => "text/css; charset=utf-8",
                    Some("svg") => "image/svg+xml",
                    Some("ico") => "image/x-icon",
                    _ => "application/octet-stream",
                };

                let response = Response::builder()
                    .header("Content-Type", mime)
                    .header("Cache-Control", "public, max-age=31536000, immutable")
                    .body(file.data.into_owned())
                    .unwrap();

                responder.respond(response);
            });
        }
    }
}

#[cfg(not(any(feature = "desktop", feature = "native")))]
mod _impl {
    use dioxus::prelude::*;

    pub const FAVICON: Asset = asset!("/assets/favicon.ico");
    pub const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
    pub const BULMA_CSS: Asset = asset!("/assets/bulma_v1.0.4.min.css");
    pub const HEADER: Asset = asset!("/assets/header.svg");
    pub const GITHUB_MARK: Asset = asset!("/assets/github-mark.svg");
    pub const PULL_REQUEST: Asset = asset!("/assets/pull-request.svg");
    pub const BRANCH: Asset = asset!("/assets/branch.svg");
    pub const SORT: Asset = asset!("/assets/sort.svg");
    pub const UP_ARROW: Asset = asset!("/assets/up-arrow.svg");
    pub const DOWN_ARROW: Asset = asset!("/assets/down-arrow.svg");
    pub const REFRESH: Asset = asset!("/assets/refresh.svg");
    pub const GEAR: Asset = asset!("/assets/gear.svg");

    pub fn install() {}
}

pub use _impl::*;
