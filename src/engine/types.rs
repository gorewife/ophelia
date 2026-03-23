pub struct DownloadInfo {
    pub id: u64,
    pub url: String,
    pub file_name: String,
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Finished,
    Error,
}

pub struct ActiveDownload {
    pub info: DownloadInfo,
    pub status: DownloadStatus,
    pub current_byte: u64,
    pub error: Option<String>,
}

impl ActiveDownload {
    pub fn progress(&self) -> f64 {
        match self.info.total_bytes {
            Some(total) if total > 0 => self.current_byte as f64 / total as f64,
            _ => 0.0,
        }
    }

    pub fn formatted_size(&self) -> String {
        format_bytes(self.info.total_bytes.unwrap_or(0))
    }
}

/// Format a byte count into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
