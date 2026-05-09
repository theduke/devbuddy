#[cfg(any(feature = "desktop", feature = "native"))]
pub async fn execute_request(
    req: http::Request<String>,
) -> Result<http::Response<Vec<u8>>, anyhow::Error> {
    fn execute_request_sync(
        req: http::Request<String>,
    ) -> Result<http::Response<Vec<u8>>, anyhow::Error> {
        let (parts, mut body) = ureq::run(req)?.into_parts();

        let body = body.read_to_vec()?;

        Ok(http::Response::from_parts(parts, body))
    }

    let (tx, rx) = futures::channel::oneshot::channel();

    std::thread::spawn(move || {
        let res = execute_request_sync(req);
        tx.send(res)
            .expect("could not send request response to channel");
    });

    rx.await?
}

#[cfg(not(any(feature = "desktop", feature = "native")))]
pub async fn execute_request(
    req: http::Request<String>,
) -> Result<http::Response<Vec<u8>>, anyhow::Error> {
    let _ = req;
    panic!("web requests not supported")
}
