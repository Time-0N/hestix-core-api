use std::time::Duration;
use anyhow::Result;
use reqwest::{Client, redirect::Policy};

pub fn build_http_client(
    user_agent: &str,
    accept_invalid_certs: bool, // only true in dev
    max_redirects: usize,
    connect_timeout: Duration,
    request_timeout: Duration,
) -> Result<Client> {
    let mut builder = Client::builder()
        .user_agent(user_agent)
        .connect_timeout(connect_timeout)
        .timeout(request_timeout)
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .redirect(Policy::limited(max_redirects))
        .gzip(true)
        .brotli(true)
        .deflate(true);

    if accept_invalid_certs {
        // DEV ONLY. Gate with an env var in bootstrap.
        builder = builder.danger_accept_invalid_certs(true);
    }

    Ok(builder.build()?)
}
