use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DownloadId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Finished,
    Error,
}

pub enum EngineCommand {
    Add { id: DownloadId, url: String, destination: PathBuf, config: DownloadConfig },
    Pause { id: DownloadId },
    Resume { id: DownloadId },
    Cancel { id: DownloadId },
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub id: DownloadId,
    pub status: DownloadStatus,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bytes_per_sec: u64,
}

pub struct DownloadConfig {
    pub max_connections: usize,
    pub write_buffer_size: usize,
    pub progress_interval_ms: u64,
    pub stall_timeout_secs: u64,
    pub max_retries_per_chunk: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_connections: 8,
            write_buffer_size: 64 * 1024,
            progress_interval_ms: 30,
            stall_timeout_secs: 100,
            max_retries_per_chunk: 5,
        }
    }
}
