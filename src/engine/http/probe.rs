//! Server capability probe.
//!
//! A single GET with `Range: bytes=0-0` tells us two things:
//!   - 206 Partial Content → server supports range requests (parallel chunks OK)
//!   - Content-Range header → total file size
//!
//! HTTP/2 is fine here since this is one request, not the parallel chunk phase.

use reqwest::StatusCode;

pub struct ProbeResult {
    pub content_length: Option<u64>,
    pub accepts_ranges: bool,
}

pub async fn probe(client: &reqwest::Client, url: &str) -> Result<ProbeResult, reqwest::Error> {
    let response = client.get(url).header("Range", "bytes=0-0").send().await?;

    if response.status() == StatusCode::PARTIAL_CONTENT {
        let total = response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split('/').last())
            .and_then(|v| v.parse::<u64>().ok());
        Ok(ProbeResult { content_length: total, accepts_ranges: true })
    } else {
        Ok(ProbeResult { content_length: response.content_length(), accepts_ranges: false })
    }
}
