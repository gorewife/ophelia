//! Per-download configuration for HTTP/HTTPS downloads.
//! Fields here are intentionally HTTP-specific: connection count, stall detection,
//! and retry behavior are concepts that don't apply to all protocols.

#[derive(Debug, Clone)]
pub struct HttpDownloadConfig {
    pub max_connections: usize,
    pub write_buffer_size: usize,
    pub progress_interval_ms: u64,
    pub stall_timeout_secs: u64,
    pub max_retries_per_chunk: u32,
    /// Minimum bytes that must remain in each half of a potential steal.
    /// A steal requires at least 2× this value remaining in the target chunk.
    /// Lowered in tests to exercise the code path on small files.
    pub min_steal_bytes: u64,
}

impl Default for HttpDownloadConfig {
    fn default() -> Self {
        Self {
            max_connections: 8,
            write_buffer_size: 64 * 1024,
            progress_interval_ms: 100,
            stall_timeout_secs: 30,
            max_retries_per_chunk: 3,
            min_steal_bytes: 4 * 1024 * 1024,
        }
    }
}
