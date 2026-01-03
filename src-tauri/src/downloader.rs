// 1a. downloader.rs - handles youtube video downloads
// 1b. wraps yt-dlp command line tool
// 1c. we shell out instead of using rust libs cuz theyre kinda mid
// also yt-dlp handles all the edge cases already

use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;
use thiserror::Error;
use crate::setup;

// 2a. error types for download failures
// thiserror saves us from writing a bunch of boilerplate
#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("yt-dlp not found - install with: pip install yt-dlp")]
    YtDlpNotFound,
    
    #[error("download failed: {0}")]
    DownloadFailed(String),
    
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type DownloadResult<T> = Result<T, DownloadError>;

// 3a. info about a downloaded video
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub path: PathBuf,
    pub title: String,
    pub duration: f64,  // seconds
    pub source_url: String,
}

// 4a. the downloader struct
pub struct Downloader {
    output_dir: PathBuf,
}

impl Downloader {
    pub fn new(output_dir: impl AsRef<Path>) -> DownloadResult<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&output_dir)?;
        Ok(Self { output_dir })
    }

    // 5a. check if yt-dlp is available (either in PATH or downloaded)
    pub fn check_ytdlp_installed() -> bool {
        setup::check_ytdlp_available()
    }
    
    // 5b. get yt-dlp command path (downloads if needed)
    async fn get_ytdlp_cmd(&self) -> DownloadResult<String> {
        setup::get_ytdlp_command().await
            .map_err(|e| DownloadError::DownloadFailed(e))
    }

    // 5b. download a single video
    // async so we can download multiple at once
    pub async fn download_video(&self, url: &str) -> DownloadResult<VideoInfo> {
        log::info!("downloading: {}", url);

        // get video info first
        let info = self.get_video_info(url).await?;
        
        // generate filename from video id
        // using uuid as backup if we cant extract id
        let video_id = extract_video_id(url).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let output_path = self.output_dir.join(format!("{}.mp4", video_id));
        
        // skip if already downloaded (nice for retries)
        if output_path.exists() {
            log::info!("already got this one: {}", output_path.display());
            return Ok(VideoInfo {
                path: output_path,
                title: info.0,
                duration: info.1,
                source_url: url.to_string(),
            });
        }

        // get yt-dlp command (auto-downloads if needed)
        let ytdlp_cmd = self.get_ytdlp_cmd().await?;
        
        // actually download with yt-dlp
        // want best quality mp4 with both video and audio
        let output = AsyncCommand::new(&ytdlp_cmd)
            .args([
                "-f", "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
                "--merge-output-format", "mp4",
                "-o", output_path.to_str().unwrap(),
                "--no-playlist",  // dont download whole playlist
                "--no-warnings",
                url,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("yt-dlp failed: {}", stderr);
            return Err(DownloadError::DownloadFailed(stderr.to_string()));
        }

        log::info!("got it: {} -> {}", url, output_path.display());

        Ok(VideoInfo {
            path: output_path,
            title: info.0,
            duration: info.1,
            source_url: url.to_string(),
        })
    }

    // 6a. get video info without downloading
    // returns (title, duration)
    async fn get_video_info(&self, url: &str) -> DownloadResult<(String, f64)> {
        let ytdlp_cmd = self.get_ytdlp_cmd().await?;
        
        let output = AsyncCommand::new(&ytdlp_cmd)
            .args([
                "--dump-json",
                "--no-download",
                "--no-warnings",
                url,
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(DownloadError::InvalidUrl(url.to_string()));
        }

        // parse the json
        // yt-dlp dumps a ton of stuff but we only need title and duration
        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| DownloadError::DownloadFailed(e.to_string()))?;

        let title = json["title"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        let duration = json["duration"]
            .as_f64()
            .unwrap_or(0.0);

        Ok((title, duration))
    }

    // 7a. download multiple videos concurrently
    // way faster than one at a time
    pub async fn download_multiple(&self, urls: &[String]) -> Vec<DownloadResult<VideoInfo>> {
        use futures::future::join_all;

        let futures: Vec<_> = urls
            .iter()
            .map(|url| self.download_video(url))
            .collect();

        join_all(futures).await
    }
}

// 8a. extract video id from youtube url
// handles all the different url formats they use
// lmao youtube has like 5 different url patterns
fn extract_video_id(url: &str) -> Option<String> {
    // youtube.com/watch?v= format
    if let Some(pos) = url.find("v=") {
        let start = pos + 2;
        let end = url[start..]
            .find('&')
            .map(|i| start + i)
            .unwrap_or(url.len());
        return Some(url[start..end].to_string());
    }
    
    // youtu.be/ format
    if let Some(pos) = url.find("youtu.be/") {
        let start = pos + 9;
        let end = url[start..]
            .find('?')
            .map(|i| start + i)
            .unwrap_or(url.len());
        return Some(url[start..end].to_string());
    }
    
    // youtube.com/shorts/ format
    if let Some(pos) = url.find("/shorts/") {
        let start = pos + 8;
        let end = url[start..]
            .find('?')
            .map(|i| start + i)
            .unwrap_or(url.len());
        return Some(url[start..end].to_string());
    }
    
    None
}

// 9a. tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        // standard watch url
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // short url
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // shorts url
        assert_eq!(
            extract_video_id("https://youtube.com/shorts/abc123"),
            Some("abc123".to_string())
        );
        
        // with extra params
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }
}
