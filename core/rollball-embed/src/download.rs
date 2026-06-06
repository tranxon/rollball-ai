//! HuggingFace model downloader with mirror support.
//!
//! Downloads ONNX model files and tokenizers from HuggingFace Hub.
//! Supports HF Mirror (e.g., `hf-mirror.com`) for users behind the GFW.
//! Uses atomic file writes (write to temp, then rename) for safety.
//!
//! # Cross-platform notes
//!
//! `std::fs::rename` behaves differently across platforms:
//! - **Unix**: atomically replaces the target if it exists.
//! - **Windows**: fails if the target already exists.
//!
//! We use a `rename_or_replace` helper that handles both cases correctly.

use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use reqwest::Client;

// ── Error type ──────────────────────────────────────────────────────────

/// Error type for download operations.
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Download cancelled")]
    Cancelled,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

// ── Cross-platform rename helper ────────────────────────────────────────

/// Rename a file or directory, replacing the target if it already exists.
///
/// On Unix, `std::fs::rename` atomically replaces the target.
/// On Windows, `std::fs::rename` fails if the target exists, so we
/// remove the target first and then rename.
fn rename_or_replace(src: &Path, dst: &Path) -> std::io::Result<()> {
    // Try the simple rename first (works on Unix and when dst doesn't exist)
    if let Ok(()) = std::fs::rename(src, dst) {
        return Ok(());
    }
    // Fallback: remove target then rename (needed on Windows when dst exists)
    if dst.exists() {
        if dst.is_dir() {
            std::fs::remove_dir_all(dst)?;
        } else {
            std::fs::remove_file(dst)?;
        }
    }
    std::fs::rename(src, dst)
}

// ── HuggingFace URL builder ─────────────────────────────────────────────

/// Build the download URL for a HuggingFace file.
///
/// Standard: `https://huggingface.co/{repo}/resolve/main/{path}`
/// Mirror:   `https://hf-mirror.com/{repo}/resolve/main/{path}`
fn hf_file_url(hf_repo: &str, file_path: &str, mirror: Option<&str>) -> String {
    let base = mirror.unwrap_or("https://huggingface.co");
    format!("{base}/{hf_repo}/resolve/main/{file_path}")
}

// ── Download result ─────────────────────────────────────────────────────

/// Result of a model download operation.
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Local directory where files were saved.
    pub model_dir: PathBuf,
    /// List of files that were downloaded.
    pub downloaded_files: Vec<String>,
}

/// Progress callback type: `(downloaded_bytes, total_bytes)`.
pub type ProgressCb = dyn Fn(u64, u64) + Send + Sync;

// ── Downloader ──────────────────────────────────────────────────────────

/// HuggingFace model downloader.
pub struct Downloader {
    /// HTTP client.
    http_client: Client,
    /// Base models directory.
    models_dir: PathBuf,
    /// Optional HF mirror domain (e.g., "https://hf-mirror.com").
    hf_mirror: Option<String>,
}

impl Downloader {
    /// Create a new downloader.
    pub fn new(models_dir: &Path, hf_mirror: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(600)) // 10 min per file
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client for downloader");

        Self {
            http_client,
            models_dir: models_dir.to_path_buf(),
            hf_mirror,
        }
    }

    /// Download a model from HuggingFace.
    ///
    /// Downloads the ONNX model file (selected variant) and tokenizer.
    /// Files are written to a temp directory first, then atomically renamed.
    ///
    /// # Arguments
    /// * `model_id` - Model identifier (used as directory name).
    /// * `hf_repo` - HuggingFace repository (e.g., "onnx-community/bge-small-zh-v1.5-ONNX").
    /// * `onnx_file` - Path within the repo to the ONNX file (e.g., "onnx/model_fp16.onnx").
    /// * `tokenizer_file` - Path within the repo to the tokenizer (e.g., "tokenizer.json").
    /// * `on_progress` - Optional progress callback.
    /// * `cancel` - Cancellation flag (if true, abort download).
    pub async fn download_model(
        &self,
        model_id: &str,
        hf_repo: &str,
        onnx_file: &str,
        tokenizer_file: &str,
        _on_progress: Option<&ProgressCb>,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<DownloadResult, DownloadError> {
        let model_dir = self.models_dir.join(model_id);
        let tmp_dir = self.models_dir.join(format!("{model_id}.downloading"));

        // Clean up any previous incomplete download
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir)?;
        }

        // Create temp directory
        std::fs::create_dir_all(&tmp_dir)?;

        let mut downloaded_files = Vec::new();

        // Files to download: (remote_path, local_name)
        // The ONNX file is always saved as "model.onnx" for consistent loading.
        // The tokenizer is always saved as "tokenizer.json".
        let files_to_download = [
            (onnx_file, "model.onnx"),
            (tokenizer_file, "tokenizer.json"),
        ];

        for (remote_path, local_name) in &files_to_download {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = std::fs::remove_dir_all(&tmp_dir);
                return Err(DownloadError::Cancelled);
            }

            let url = hf_file_url(hf_repo, remote_path, self.hf_mirror.as_deref());
            let local_path = tmp_dir.join(local_name);

            tracing::info!(url = %url, local = %local_path.display(), "Downloading file");

            self.download_file(&url, &local_path).await?;

            downloaded_files.push(local_name.to_string());
        }

        // Download ONNX external data file if present.
        // Many ONNX models use external data (e.g., model_fp16.onnx_data) where
        // the ONNX file contains the graph structure and the actual weights are
        // in a companion file. The external data filename is referenced inside
        // the ONNX file (e.g., "model_fp16.onnx_data"), so we must preserve
        // the original filename from the HF repo but strip the directory prefix.
        // ONNX Runtime resolves external data relative to the directory containing
        // model.onnx, using the bare filename stored in the protobuf.
        let onnx_ext_data_remote = format!("{}_data", onnx_file);
        let onnx_ext_data_local = Path::new(&onnx_ext_data_remote)
            .file_name()
            .expect("onnx_ext_data_remote should have a filename")
            .to_str()
            .expect("filename should be valid UTF-8")
            .to_string();
        let ext_data_url = hf_file_url(hf_repo, &onnx_ext_data_remote, self.hf_mirror.as_deref());
        let ext_data_path = tmp_dir.join(&onnx_ext_data_local);

        tracing::info!(url = %ext_data_url, local = %ext_data_path.display(), "Downloading external data file");

        match self.download_file(&ext_data_url, &ext_data_path).await {
            Ok(()) => {
                downloaded_files.push(onnx_ext_data_local);
            }
            Err(e) => {
                // External data is optional — some models embed weights directly
                tracing::info!(
                    path = %onnx_ext_data_remote,
                    error = %e,
                    "External data file not found (model may have embedded weights)"
                );
            }
        }

        // Atomic rename: tmp_dir → model_dir (cross-platform)
        rename_or_replace(&tmp_dir, &model_dir)?;

        tracing::info!(
            model_id,
            dir = %model_dir.display(),
            files = ?downloaded_files,
            "Model download complete"
        );

        Ok(DownloadResult {
            model_dir,
            downloaded_files,
        })
    }

    /// Download a single file with retry logic.
    /// Retries up to 3 times on transient HTTP errors (5xx, timeout).
    async fn download_file(
        &self,
        url: &str,
        dest: &Path,
    ) -> Result<(), DownloadError> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 2000;

        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.download_file_inner(url, dest).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    let is_transient = match &e {
                        DownloadError::Http(req_err) => {
                            req_err.is_timeout()
                                || req_err.is_connect()
                                || req_err.status().map_or(false, |s| s.is_server_error())
                        }
                        _ => false,
                    };

                    if is_transient && attempt < MAX_RETRIES {
                        tracing::warn!(
                            url,
                            attempt,
                            error = %e,
                            "Download failed (transient), retrying..."
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(
                            RETRY_DELAY_MS * attempt as u64,
                        ))
                        .await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Inner download implementation (single attempt).
    /// Uses streaming to avoid loading the entire file into memory.
    async fn download_file_inner(
        &self,
        url: &str,
        dest: &Path,
    ) -> Result<(), DownloadError> {
        let response = self
            .http_client
            .get(url)
            .send()
            .await?
            .error_for_status()
            .map_err(DownloadError::Http)?;

        let total_size = response.content_length().unwrap_or(0);
        tracing::info!(size = total_size, "Downloading file");

        // Write to temp file first via streaming, then atomic rename (cross-platform)
        let tmp_path = dest.with_extension("tmp");
        let mut file = tokio::io::BufWriter::new(tokio::fs::File::create(&tmp_path).await?);

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(DownloadError::Http)?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
        }
        tokio::io::AsyncWriteExt::flush(&mut file).await?;
        drop(file);

        rename_or_replace(&tmp_path, dest)?;

        Ok(())
    }

    /// Check if a model is already downloaded.
    pub fn is_downloaded(&self, model_id: &str) -> bool {
        let model_dir = self.models_dir.join(model_id);
        model_dir.exists()
            && model_dir.join("model.onnx").exists()
            && model_dir.join("tokenizer.json").exists()
    }

    /// Delete a downloaded model.
    pub fn delete_model(&self, model_id: &str) -> Result<(), DownloadError> {
        let model_dir = self.models_dir.join(model_id);
        if model_dir.exists() {
            std::fs::remove_dir_all(&model_dir)?;
            tracing::info!(model_id, "Deleted model files");
        }
        Ok(())
    }

    /// Get the models directory path.
    pub fn models_dir_path(&self) -> &Path {
        &self.models_dir
    }
}
