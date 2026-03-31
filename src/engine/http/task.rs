//! HTTP/HTTPS download pipeline.
//!
//! Probes server capabilities via GET+Range, then either drives parallel
//! chunked range requests (206) or falls back to a single stream (200).
//! Progress is tracked via per-chunk atomics; the timer starts after chunks
//! are spawned to exclude probe and allocation time from speed calculations.
//!
//! Pause/resume: a `CancellationToken` is shared with every chunk task.
//! Each chunk checks it at the top of its byte loop via `biased select!` so
//! pause takes effect within one loop iteration. On pause the write buffer is
//! flushed before returning so saved offsets match bytes actually on disk.

use std::collections::{HashSet, VecDeque};
use std::sync::atomic::AtomicUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::StreamExt;
use reqwest::StatusCode;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::engine::chunk;
use crate::engine::http::HttpDownloadConfig;
use crate::engine::types::{ChunkSnapshot, DownloadId, DownloadStatus, ProgressUpdate};

// --- error classification -----------------------------------------------

/// Classification of chunk-level errors, used to decide whether to retry.
enum ChunkError {
    /// Transient failure. `retry_after` is populated from the Retry-After header on 429.
    Retryable { retry_after: Option<Duration> },
    /// Server refused definitively (403, 404, 410). Retrying won't help.
    NonRetryable,
    /// Local failure (disk full, permission denied). Stops the entire download.
    Fatal(String),
    /// Soft pause requested via CancellationToken — exit cleanly, save state.
    Paused,
    /// Health monitor killed this connection (too slow) — retry on a fresh connection.
    Killed,
}

fn classify_status(status: StatusCode, headers: &reqwest::header::HeaderMap) -> ChunkError {
    match status.as_u16() {
        403 | 404 | 410 => ChunkError::NonRetryable,
        429 => {
            let retry_after = headers
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(Duration::from_secs);
            ChunkError::Retryable { retry_after }
        }
        500..=599 => ChunkError::Retryable { retry_after: None },
        _ => ChunkError::NonRetryable,
    }
}

fn classify_io_error(e: std::io::Error) -> ChunkError {
    match e.kind() {
        std::io::ErrorKind::StorageFull | std::io::ErrorKind::PermissionDenied => {
            ChunkError::Fatal(e.to_string())
        }
        _ => ChunkError::Retryable { retry_after: None },
    }
}

// --- outcome reported per chunk to the outer drain loop -----------------

enum ChunkOutcome {
    Finished,
    Paused,
    Failed,
}

// --- server probe --------------------------------------------------------

struct ProbeResult {
    content_length: Option<u64>,
    accepts_ranges: bool,
}

async fn probe(client: &reqwest::Client, url: &str) -> Result<ProbeResult, reqwest::Error> {
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

// --- main entry point ----------------------------------------------------

/// `pause_sink`: written by this task on soft pause so the engine can read chunk
/// offsets for resume. `resume_from`: `Some` when resuming a previously paused download.
#[tracing::instrument(name = "download", skip(config, progress_tx, pause_token, pause_sink, resume_from), fields(id = id.0, %url))]
pub async fn download_task(
    id: DownloadId,
    url: String,
    destination: PathBuf,
    config: HttpDownloadConfig,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
    pause_token: CancellationToken,
    pause_sink: Arc<Mutex<Option<Vec<ChunkSnapshot>>>>,
    resume_from: Option<Vec<ChunkSnapshot>>,
) {
    let send = |status: DownloadStatus, downloaded: u64, total: Option<u64>, speed: u64| {
        let _ = progress_tx.send(ProgressUpdate {
            id,
            status,
            downloaded_bytes: downloaded,
            total_bytes: total,
            speed_bytes_per_sec: speed,
        });
    };

    // Probe uses the default client (HTTP/2 fine for a single HEAD-like request).
    // Chunk downloads use an HTTP/1.1-only client so each range request gets its
    // own TCP connection. HTTP/2 multiplexes everything onto one connection, which
    // defeats parallel chunking entirely — Surge disables H2 for the same reason.
    let probe_client = reqwest::Client::new();
    let chunk_client = Arc::new(
        reqwest::Client::builder()
            .http1_only()
            .build()
            .expect("failed to build HTTP/1.1 client"),
    );

    // Partial file uses a .ophelia_part suffix so other apps don't try to open it
    // mid-download. On completion we rename to the final destination atomically.
    let part_path = {
        let mut p = destination.clone();
        let name = p
            .file_name()
            .map(|n| format!("{}.ophelia_part", n.to_string_lossy()))
            .unwrap_or_else(|| "download.ophelia_part".into());
        p.set_file_name(name);
        p
    };

    // 1. Resolve total size, chunk boundaries, and file handle.
    //    On resume: restore from snapshot, open existing .ophelia_part file (no truncation).
    //    On fresh start: probe server, allocate file, split into chunks.
    let (total_bytes, chunks, file) = match resume_from {
        Some(snapshots) => {
            let total = snapshots.last().map(|s| s.end).unwrap_or(0);
            let cl = chunk::ChunkList {
                starts: snapshots.iter().map(|s| s.start).collect(),
                ends: snapshots.iter().map(|s| s.end).collect(),
                downloaded: snapshots.iter().map(|s| s.downloaded).collect(),
                statuses: snapshots
                    .iter()
                    .map(|s| {
                        if s.downloaded >= s.end - s.start {
                            chunk::ChunkStatus::Finished
                        } else {
                            chunk::ChunkStatus::Pending
                        }
                    })
                    .collect(),
            };
            let file = match std::fs::OpenOptions::new().write(true).open(&part_path) {
                Ok(f) => f,
                Err(_) => {
                    send(DownloadStatus::Error, 0, Some(total), 0);
                    return;
                }
            };
            tracing::info!(total_bytes = total, chunks = cl.len(), "resuming chunked download");
            (total, cl, file)
        }
        None => {
            let probe_result = match probe(&probe_client, &url).await {
                Ok(p) => p,
                Err(_) => {
                    send(DownloadStatus::Error, 0, None, 0);
                    return;
                }
            };
            tracing::debug!(
                accepts_ranges = probe_result.accepts_ranges,
                content_length = probe_result.content_length,
                "probe complete"
            );

            let total_bytes = match probe_result.content_length {
                Some(len) => len,
                None => {
                    tracing::info!("no content-length, falling back to single stream");
                    single_download(
                        id,
                        Arc::clone(&chunk_client),
                        url,
                        part_path,
                        destination,
                        config.stall_timeout_secs,
                        progress_tx,
                    )
                    .await;
                    return;
                }
            };

            let file = match std::fs::File::create(&part_path) {
                Ok(f) => f,
                Err(_) => {
                    send(DownloadStatus::Error, 0, Some(total_bytes), 0);
                    return;
                }
            };
            if file.set_len(total_bytes).is_err() {
                send(DownloadStatus::Error, 0, Some(total_bytes), 0);
                return;
            }

            let num_chunks =
                if probe_result.accepts_ranges { config.max_connections } else { 1 };
            tracing::info!(total_bytes, num_chunks, "starting chunked download");
            let chunks = chunk::split(total_bytes, num_chunks);
            (total_bytes, chunks, file)
        }
    };

    let file = Arc::new(file);

    // 2. Extract config values before moving into closures.
    let write_buffer_size = config.write_buffer_size;
    let progress_interval_ms = config.progress_interval_ms;
    let stall_timeout = Duration::from_secs(config.stall_timeout_secs);
    let max_retries = config.max_retries_per_chunk;
    let min_steal_bytes = config.min_steal_bytes;
    let num_initial_chunks = chunks.len();

    // 3. Per-chunk atomics, pre-allocated with headroom for stolen chunks.
    //
    //    Work stealing adds new slots at runtime. The steal budget is bounded: at most
    //    num_initial_chunks steals can ever happen (one new chunk per initial chunk),
    //    so pre-allocating num_initial_chunks extra slots is always sufficient.
    //
    //    chunk_starts_atomic and chunk_ends_atomic are writable so a steal can shrink a
    //    victim's end and assign the remainder to a new slot. make_chunk_fut reads them
    //    from the atomics on each retry so it sees the updated boundary without restart.
    let steal_budget = num_initial_chunks;
    let total_slots = num_initial_chunks + steal_budget;

    let chunk_starts_atomic: Arc<Vec<AtomicU64>> = Arc::new(
        chunks.starts.iter().map(|&s| AtomicU64::new(s))
            .chain((0..steal_budget).map(|_| AtomicU64::new(0)))
            .collect(),
    );
    let chunk_ends_atomic: Arc<Vec<AtomicU64>> = Arc::new(
        chunks.ends.iter().map(|&e| AtomicU64::new(e))
            .chain((0..steal_budget).map(|_| AtomicU64::new(0)))
            .collect(),
    );
    let chunk_downloaded: Arc<Vec<AtomicU64>> = Arc::new(
        chunks.downloaded.iter().map(|&d| AtomicU64::new(d))
            .chain((0..steal_budget).map(|_| AtomicU64::new(0)))
            .collect(),
    );

    // Per-slot kill tokens: wrapped in Mutex so each attempt can swap in a fresh
    // CancellationToken. CancellationToken is a one-shot signal — once cancelled
    // it cannot be reset, so we replace the whole token at the start of each attempt.
    let kill_tokens: Arc<Vec<Mutex<CancellationToken>>> = Arc::new(
        (0..total_slots).map(|_| Mutex::new(CancellationToken::new())).collect(),
    );

    // How many bytes each slot had downloaded when it was last activated.
    // The health monitor uses this to enforce a 1MB grace period before a slot
    // is eligible for killing — avoids killing during connection setup.
    let slot_activation: Arc<Vec<AtomicU64>> = Arc::new(
        (0..total_slots).map(|_| AtomicU64::new(0)).collect(),
    );

    // Points to the next unused slot; incremented by try_steal.
    let next_slot = Arc::new(AtomicUsize::new(num_initial_chunks));

    // 4. Queue only unfinished chunks (on a fresh start all are Pending).
    let already_done: u64 = chunk_downloaded[..num_initial_chunks]
        .iter()
        .map(|a| a.load(Ordering::Relaxed))
        .sum();
    send(DownloadStatus::Downloading, already_done, Some(total_bytes), 0);

    let mut pending: VecDeque<usize> = (0..num_initial_chunks)
        .filter(|&i| chunks.statuses[i] != chunk::ChunkStatus::Finished)
        .collect();

    // active tracks which slot indices are currently running in the JoinSet.
    // active_shared mirrors it and is accessible to the health monitor task.
    let mut active: HashSet<usize> = HashSet::new();
    let active_shared: Arc<Mutex<HashSet<usize>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut join_set: JoinSet<(usize, ChunkOutcome)> = JoinSet::new();
    let mut current_limit: usize = 1;
    let mut all_ok = true;

    // 5. Closure that builds the retry loop for slot `i`.
    //    Reads start and end from atomics each attempt so a steal that shrinks
    //    the victim's end is picked up at the next retry without restarting.
    let make_chunk_fut = |i: usize| {
        let url = url.clone();
        let client = Arc::clone(&chunk_client);
        let file = Arc::clone(&file);
        let counters = Arc::clone(&chunk_downloaded);
        let starts = Arc::clone(&chunk_starts_atomic);
        let ends = Arc::clone(&chunk_ends_atomic);
        let kills = Arc::clone(&kill_tokens);
        let activation = Arc::clone(&slot_activation);
        let pause_token = pause_token.clone();
        async move {
            let mut attempt = 0u32;
            loop {
                // Fresh kill token for this attempt. We replace the token stored in the
                // Mutex so the health monitor always cancels the current connection.
                // The old token (from the previous attempt) is simply dropped.
                let kill_token = {
                    let new = CancellationToken::new();
                    *kills[i].lock().unwrap() = new.clone();
                    new
                };
                // Snapshot downloaded bytes so the health monitor's grace period
                // measures progress from the start of this attempt, not all time.
                activation[i].store(counters[i].load(Ordering::Relaxed), Ordering::Relaxed);

                let start = starts[i].load(Ordering::Acquire);
                let end = ends[i].load(Ordering::Acquire);
                let resume_from = counters[i].load(Ordering::Relaxed);
                match download_chunk(
                    &client,
                    &url,
                    start,
                    end,
                    resume_from,
                    &file,
                    &counters,
                    i,
                    write_buffer_size,
                    stall_timeout,
                    &pause_token,
                    &kill_token,
                )
                .await
                {
                    Ok(()) => return (i, ChunkOutcome::Finished),
                    Err(ChunkError::Paused) => return (i, ChunkOutcome::Paused),
                    Err(ChunkError::Killed) => {
                        tracing::debug!(chunk = i, "slow worker killed, retrying");
                        continue;
                    }
                    Err(ChunkError::Fatal(msg)) => {
                        tracing::error!(chunk = i, msg, "fatal chunk error");
                        return (i, ChunkOutcome::Failed);
                    }
                    Err(ChunkError::NonRetryable) => {
                        tracing::error!(chunk = i, "non-retryable server error");
                        return (i, ChunkOutcome::Failed);
                    }
                    Err(ChunkError::Retryable { retry_after }) => {
                        if counters[i].load(Ordering::Relaxed) > resume_from {
                            attempt = 0;
                        } else {
                            attempt += 1;
                        }
                        if attempt >= max_retries {
                            tracing::error!(chunk = i, attempt, "max retries exceeded");
                            return (i, ChunkOutcome::Failed);
                        }
                        let delay = retry_after.unwrap_or_else(|| {
                            Duration::from_secs(2u64.pow(attempt.min(5)).min(30))
                        });
                        tracing::warn!(
                            chunk = i,
                            attempt,
                            delay_secs = delay.as_secs(),
                            "retrying chunk"
                        );
                        tokio::select! {
                            biased;
                            _ = pause_token.cancelled() => return (i, ChunkOutcome::Paused),
                            _ = tokio::time::sleep(delay) => {}
                        }
                    }
                }
            }
        }
    };

    if let Some(i) = pending.pop_front() {
        active.insert(i);
        active_shared.lock().unwrap().insert(i);
        join_set.spawn(make_chunk_fut(i));
    }

    // 6. Background progress reporter with EMA speed.
    //
    //    Simple total/elapsed is biased toward the start of the download and spikes
    //    wildly on resume. Instead we use a 2-second sliding window EMA:
    //    speed = (1 - α) * speed + α * recent_window_speed   (α = 0.3)
    //    On stall (no bytes for > window), speed decays proportionally so the display
    //    reflects that nothing is happening rather than showing the last good value.
    let progress_handle = {
        let counters = Arc::clone(&chunk_downloaded);
        let slot_count = Arc::clone(&next_slot);
        let progress_tx = progress_tx.clone();
        tokio::spawn(async move {
            const EMA_ALPHA: f64 = 0.3;
            const WINDOW_SECS: f64 = 2.0;

            let mut ema_speed: f64 = 0.0;
            let mut window_start = Instant::now();
            let mut window_bytes: u64 = 0;
            let mut last_total: u64 = already_done;

            loop {
                tokio::time::sleep(Duration::from_millis(progress_interval_ms)).await;

                // Only sum populated slots (initial + any stolen).
                let populated = slot_count.load(Ordering::Relaxed);
                let total_downloaded: u64 =
                    counters[..populated].iter().map(|a| a.load(Ordering::Relaxed)).sum();
                let new_bytes = total_downloaded.saturating_sub(last_total);
                last_total = total_downloaded;
                window_bytes += new_bytes;

                let window_elapsed = window_start.elapsed().as_secs_f64();
                if window_elapsed >= WINDOW_SECS {
                    let recent = window_bytes as f64 / window_elapsed;
                    ema_speed = (1.0 - EMA_ALPHA) * ema_speed + EMA_ALPHA * recent;
                    // Decay toward zero if the window stalled
                    if window_bytes == 0 {
                        ema_speed *= WINDOW_SECS / window_elapsed.max(WINDOW_SECS);
                    }
                    window_bytes = 0;
                    window_start = Instant::now();
                }

                let _ = progress_tx.send(ProgressUpdate {
                    id,
                    status: DownloadStatus::Downloading,
                    downloaded_bytes: total_downloaded,
                    total_bytes: Some(total_bytes),
                    speed_bytes_per_sec: ema_speed as u64,
                });
                if total_downloaded >= total_bytes {
                    break;
                }
            }
        })
    };

    // 6b. Health monitor: kills connections that fall below 50% of mean speed.
    //     Ticks every 1 second. A slot must have downloaded >= 1MB since its last
    //     activation before it becomes eligible (guards against killing during TCP
    //     slow-start). Requires >= 2 eligible slots to have a meaningful mean.
    let health_handle = {
        let counters = Arc::clone(&chunk_downloaded);
        let kills = Arc::clone(&kill_tokens);
        let active = Arc::clone(&active_shared);
        let activation = Arc::clone(&slot_activation);
        let pause_token = pause_token.clone();
        tokio::spawn(async move {
            const GRACE_BYTES: u64 = 1 * 1024 * 1024;
            const SLOW_FACTOR: f64 = 0.5;
            // EMA smoothing for per-slot speed, same α as the progress reporter.
            // Raw per-second deltas are noisy (a single slow tick can trigger a kill);
            // EMA gives the same stable view that aria2 uses for its SpeedCalc.
            const EMA_ALPHA: f64 = 0.3;

            let mut prev: Vec<u64> = vec![0u64; counters.len()];
            let mut ema: Vec<f64> = vec![0.0f64; counters.len()];

            loop {
                tokio::select! {
                    biased;
                    _ = pause_token.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                }

                let active_set = active.lock().unwrap().clone();
                if active_set.len() < 2 {
                    continue;
                }

                // Update EMA for each active slot.
                for &i in &active_set {
                    let current = counters[i].load(Ordering::Relaxed);
                    let delta = current.saturating_sub(prev[i]) as f64;
                    prev[i] = current;
                    ema[i] = (1.0 - EMA_ALPHA) * ema[i] + EMA_ALPHA * delta;
                }

                // Only consider slots past the grace period (avoids killing during TCP slow-start)
                let eligible: Vec<(usize, f64)> = active_set.iter()
                    .filter(|&&i| {
                        counters[i].load(Ordering::Relaxed)
                            .saturating_sub(activation[i].load(Ordering::Relaxed))
                            >= GRACE_BYTES
                    })
                    .map(|&i| (i, ema[i]))
                    .collect();

                if eligible.len() < 2 {
                    continue;
                }

                let sum: f64 = eligible.iter().map(|(_, s)| s).sum();
                let mean = sum / eligible.len() as f64;
                if mean < 1.0 {
                    continue;
                }

                for (i, speed) in &eligible {
                    if speed < &(mean * SLOW_FACTOR) {
                        tracing::debug!(
                            slot = i, speed = *speed as u64, mean = mean as u64,
                            "health monitor killing slow worker"
                        );
                        kills[*i].lock().unwrap().cancel();
                    }
                }
            }
        })
    };

    // 7. Drain loop: ramp concurrency, try work stealing when a slot finishes.
    let mut paused = false;
    while let Some(result) = join_set.join_next().await {
        let (finished_i, outcome) = match result {
            Ok(pair) => pair,
            Err(_panic) => { all_ok = false; continue; }
        };
        active.remove(&finished_i);
        active_shared.lock().unwrap().remove(&finished_i);

        match outcome {
            ChunkOutcome::Finished if !paused => {
                current_limit = (current_limit * 2).min(total_slots);
                // When pending is empty and workers are idle, try splitting the
                // largest active chunk rather than leaving workers idle
                if pending.is_empty() {
                    try_steal(
                        &chunk_starts_atomic,
                        &chunk_ends_atomic,
                        &chunk_downloaded,
                        &active,
                        &next_slot,
                        &mut pending,
                        write_buffer_size as u64,
                        min_steal_bytes,
                    );
                }
            }
            ChunkOutcome::Paused => { paused = true; }
            ChunkOutcome::Failed if !paused => { all_ok = false; }
            _ => {}
        }
        if !paused {
            while join_set.len() < current_limit {
                if let Some(i) = pending.pop_front() {
                    active.insert(i);
                    active_shared.lock().unwrap().insert(i);
                    join_set.spawn(make_chunk_fut(i));
                } else {
                    break;
                }
            }
        }
    }

    progress_handle.abort();
    health_handle.abort();
    drop(file);
    // must close before rename on Windows

    if paused {
        // Snapshot all populated slots (initial + any stolen chunks).
        // Offsets match what's on disk because download_chunk flushes its write
        // buffer before returning Paused.
        let populated = next_slot.load(Ordering::Relaxed);
        let snapshots: Vec<ChunkSnapshot> = (0..populated)
            .map(|i| ChunkSnapshot {
                start: chunk_starts_atomic[i].load(Ordering::Relaxed),
                end: chunk_ends_atomic[i].load(Ordering::Relaxed),
                downloaded: chunk_downloaded[i].load(Ordering::Relaxed),
            })
            .collect();
        *pause_sink.lock().unwrap() = Some(snapshots);
        let total_downloaded: u64 = chunk_downloaded[..populated]
            .iter()
            .map(|a| a.load(Ordering::Relaxed))
            .sum();
        tracing::info!(total_downloaded, total_bytes, "download paused");
        send(DownloadStatus::Paused, total_downloaded, Some(total_bytes), 0);
    } else if all_ok {
        match std::fs::rename(&part_path, &destination) {
            Ok(()) => {
                tracing::info!(total_bytes, "download finished");
                send(DownloadStatus::Finished, total_bytes, Some(total_bytes), 0);
            }
            Err(e) => {
                tracing::error!(err = %e, "rename failed after download");
                send(DownloadStatus::Error, total_bytes, Some(total_bytes), 0);
            }
        }
    } else {
        let populated = next_slot.load(Ordering::Relaxed);
        let total_downloaded: u64 = chunk_downloaded[..populated]
            .iter()
            .map(|a| a.load(Ordering::Relaxed))
            .sum();
        tracing::error!(total_downloaded, total_bytes, "download failed");
        send(DownloadStatus::Error, total_downloaded, Some(total_bytes), 0);
    }
}

// --- work stealing -------------------------------------------------------

/// Splits the active chunk with the most remaining bytes, creating a new slot
/// in the pre-allocated arrays that covers the back half of the stolen range.
///
/// The stolen split point is aligned to 4KB to avoid mid-block Range requests.
/// A safe zone of `safe_zone` bytes ahead of the victim's current position is
/// excluded from stealing. These bytes may be buffered in the victim's write
/// buffer and would produce a harmless but wasteful double-write.
///
/// Minimum stealable range: 4MB in each half (8MB total remaining).
fn try_steal(
    starts: &[AtomicU64],
    ends: &[AtomicU64],
    downloaded: &[AtomicU64],
    active: &HashSet<usize>,
    next_slot: &AtomicUsize,
    pending: &mut VecDeque<usize>,
    safe_zone: u64,
    min_steal_bytes: u64,
) {
    const ALIGN: u64 = 4096;

    // Find the active slot with the most stealable bytes.
    let victim = active
        .iter()
        .filter_map(|&i| {
            let start = starts[i].load(Ordering::Relaxed);
            let end = ends[i].load(Ordering::Relaxed);
            let dl = downloaded[i].load(Ordering::Relaxed);
            let current = start + dl;
            // Exclude bytes the victim may have buffered but not yet flushed.
            let stealable_start = (current + safe_zone).min(end);
            let stealable = end.saturating_sub(stealable_start);
            if stealable >= 2 * min_steal_bytes {
                Some((i, stealable_start, stealable, end))
            } else {
                None
            }
        })
        .max_by_key(|&(_, _, stealable, _)| stealable);

    let (victim_i, stealable_start, stealable, victim_end) = match victim {
        Some(v) => v,
        None => return,
    };

    // Steal the back half, aligned to 4KB.
    let raw_midpoint = stealable_start + stealable / 2;
    let midpoint = (raw_midpoint + ALIGN - 1) / ALIGN * ALIGN;
    if midpoint >= victim_end || victim_end - midpoint < min_steal_bytes {
        return;
    }

    let slot = next_slot.fetch_add(1, Ordering::Relaxed);
    if slot >= starts.len() {
        next_slot.fetch_sub(1, Ordering::Relaxed);
        return; // pre-allocated budget exhausted
    }

    // Shrink victim's end, then initialise the new slot.
    // Release on the store pairs with Acquire in make_chunk_fut so the victim
    // sees the new boundary on its next retry.
    ends[victim_i].store(midpoint, Ordering::Release);
    starts[slot].store(midpoint, Ordering::Relaxed);
    ends[slot].store(victim_end, Ordering::Relaxed);
    // downloaded[slot] starts at 0 (pre-initialised in the main array).

    pending.push_front(slot);
    tracing::debug!(victim = victim_i, slot, midpoint, stolen_bytes = victim_end - midpoint, "work stolen");
}

// --- chunk worker --------------------------------------------------------

async fn download_chunk(
    client: &reqwest::Client,
    url: &str,
    chunk_start: u64,
    chunk_end: u64,
    resume_from: u64,
    file: &std::fs::File,
    counters: &[AtomicU64],
    index: usize,
    write_buffer_size: usize,
    stall_timeout: Duration,
    pause_token: &CancellationToken,
    kill_token: &CancellationToken,
) -> Result<(), ChunkError> {
    let byte_start = chunk_start + resume_from;
    // After a work steal, the victim re-enters with byte_start >= its new (shrunk) end.
    // The bytes are already on disk (written in the previous request); return early
    // TODO: I'm scared.
    if byte_start >= chunk_end {
        return Ok(());
    }
    let range = format!("bytes={}-{}", byte_start, chunk_end - 1);

    let response = client
        .get(url)
        .header("Range", range)
        .send()
        .await
        .map_err(|_| ChunkError::Retryable { retry_after: None })?;

    if !response.status().is_success() {
        return Err(classify_status(response.status(), response.headers()));
    }

    let mut stream = response.bytes_stream();
    let mut offset = byte_start;
    let mut buffer = Vec::with_capacity(write_buffer_size);

    loop {
        tokio::select! {
            // Pause is checked first (biased) so it takes effect within one
            // loop iteration regardless of network activity.
            biased;
            _ = pause_token.cancelled() => {
                if !buffer.is_empty() {
                    write_at(file, &buffer, offset).map_err(classify_io_error)?;
                    counters[index].fetch_add(buffer.len() as u64, Ordering::Relaxed);
                }
                return Err(ChunkError::Paused);
            }
            _ = kill_token.cancelled() => {
                // Health monitor determined this connection is too slow. Flush so
                // progress isn't lost, then return Killed. The retry loop opens a
                // fresh connection immediately without counting against the retry budget.
                if !buffer.is_empty() {
                    write_at(file, &buffer, offset).map_err(classify_io_error)?;
                    counters[index].fetch_add(buffer.len() as u64, Ordering::Relaxed);
                }
                return Err(ChunkError::Killed);
            }
            result = tokio::time::timeout(stall_timeout, stream.next()) => {
                match result {
                    Err(_elapsed) => return Err(ChunkError::Retryable { retry_after: None }),
                    Ok(None) => break,
                    Ok(Some(Err(_))) => return Err(ChunkError::Retryable { retry_after: None }),
                    Ok(Some(Ok(bytes))) => {
                        buffer.extend_from_slice(&bytes);
                        if buffer.len() >= write_buffer_size {
                            write_at(file, &buffer, offset).map_err(classify_io_error)?;
                            offset += buffer.len() as u64;
                            counters[index].fetch_add(buffer.len() as u64, Ordering::Relaxed);
                            buffer.clear();
                        }
                    }
                }
            }
        }
    }

    if !buffer.is_empty() {
        write_at(file, &buffer, offset).map_err(classify_io_error)?;
        counters[index].fetch_add(buffer.len() as u64, Ordering::Relaxed);
    }

    Ok(())
}

// --- single-stream fallback (no Content-Length) -------------------------

/// **No parallel chunks & no resume**
/// Used when the server omits Content-Length.
async fn single_download(
    id: DownloadId,
    client: Arc<reqwest::Client>,
    url: String,
    part_path: PathBuf,
    destination: PathBuf,
    stall_timeout_secs: u64,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) {
    let stall_timeout = Duration::from_secs(stall_timeout_secs);

    let send = |status: DownloadStatus, downloaded: u64, total: Option<u64>, speed: u64| {
        let _ = progress_tx.send(ProgressUpdate {
            id,
            status,
            downloaded_bytes: downloaded,
            total_bytes: total,
            speed_bytes_per_sec: speed,
        });
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => {
            send(DownloadStatus::Error, 0, None, 0);
            return;
        }
    };

    let mut file = match tokio::fs::File::create(&part_path).await {
        Ok(f) => f,
        Err(_) => {
            send(DownloadStatus::Error, 0, None, 0);
            return;
        }
    };

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    send(DownloadStatus::Downloading, 0, None, 0);

    const EMA_ALPHA: f64 = 0.3;
    const WINDOW_SECS: f64 = 2.0;
    let mut ema_speed: f64 = 0.0;
    let mut window_start = Instant::now();
    let mut window_bytes: u64 = 0;

    loop {
        match tokio::time::timeout(stall_timeout, stream.next()).await {
            Err(_) | Ok(Some(Err(_))) => {
                send(DownloadStatus::Error, downloaded, None, 0);
                return;
            }
            Ok(None) => break,
            Ok(Some(Ok(chunk))) => {
                if tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await.is_err() {
                    send(DownloadStatus::Error, downloaded, None, 0);
                    return;
                }
                downloaded += chunk.len() as u64;
                window_bytes += chunk.len() as u64;
                let window_elapsed = window_start.elapsed().as_secs_f64();
                if window_elapsed >= WINDOW_SECS {
                    let recent = window_bytes as f64 / window_elapsed;
                    ema_speed = (1.0 - EMA_ALPHA) * ema_speed + EMA_ALPHA * recent;
                    window_bytes = 0;
                    window_start = Instant::now();
                }
                send(DownloadStatus::Downloading, downloaded, None, ema_speed as u64);
            }
        }
    }

    drop(file);
    match std::fs::rename(&part_path, &destination) {
        Ok(()) => send(DownloadStatus::Finished, downloaded, None, 0),
        Err(e) => {
            tracing::error!(err = %e, "rename failed after single download");
            send(DownloadStatus::Error, downloaded, None, 0);
        }
    }
}

// --- platform write-at ---------------------------------------------------

#[cfg(unix)]
fn write_at(file: &std::fs::File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    file.write_all_at(buf, offset)
}

#[cfg(windows)]
fn write_at(file: &std::fs::File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    file.seek_write(buf, offset)?;
    Ok(())
}
