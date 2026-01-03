// 1a. setup.rs - auto-download yt-dlp if user doesnt have it
// 1b. downloads from github releases on first run
// 1c. saves to app data dir so it persists
// this is way better than making users install stuff manually

use std::path::PathBuf;
use tokio::fs;

// 2a. ensure yt-dlp is available
// checks PATH first, then app data dir, downloads if needed
pub async fn ensure_ytdlp() -> Result<PathBuf, String> {
    // first check if already in PATH
    if let Ok(path) = which::which("yt-dlp") {
        log::info!("found yt-dlp in PATH: {}", path.display());
        return Ok(path);
    }
    
    // check our app data directory
    let app_dir = get_app_bin_dir()?;
    let ytdlp_path = get_ytdlp_path(&app_dir);
    
    if ytdlp_path.exists() {
        log::info!("found bundled yt-dlp: {}", ytdlp_path.display());
        return Ok(ytdlp_path);
    }
    
    // gotta download it
    log::info!("yt-dlp not found, downloading...");
    download_ytdlp(&app_dir).await?;
    
    Ok(ytdlp_path)
}

// 2b. check if yt-dlp is available (without downloading)
pub fn check_ytdlp_available() -> bool {
    // check PATH
    if which::which("yt-dlp").is_ok() {
        return true;
    }
    
    // check app data dir
    if let Ok(app_dir) = get_app_bin_dir() {
        let ytdlp_path = get_ytdlp_path(&app_dir);
        if ytdlp_path.exists() {
            return true;
        }
    }
    
    false
}

// 3a. get the app's bin directory
fn get_app_bin_dir() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir()
        .ok_or("couldnt find data directory")?;
    
    Ok(data_dir.join("godz").join("bin"))
}

// 3b. get platform-specific yt-dlp path
fn get_ytdlp_path(app_dir: &PathBuf) -> PathBuf {
    if cfg!(windows) {
        app_dir.join("yt-dlp.exe")
    } else {
        app_dir.join("yt-dlp")
    }
}

// 4a. download yt-dlp from github
async fn download_ytdlp(app_dir: &PathBuf) -> Result<(), String> {
    // create the directory
    fs::create_dir_all(app_dir).await
        .map_err(|e| format!("failed to create bin dir: {}", e))?;
    
    // figure out which binary to download
    let (url, filename) = get_ytdlp_download_info();
    
    log::info!("downloading yt-dlp from: {}", url);
    
    // download it
    let response = reqwest::get(url).await
        .map_err(|e| format!("download failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("download failed with status: {}", response.status()));
    }
    
    let bytes = response.bytes().await
        .map_err(|e| format!("failed to read download: {}", e))?;
    
    let ytdlp_path = app_dir.join(filename);
    
    fs::write(&ytdlp_path, &bytes).await
        .map_err(|e| format!("failed to write file: {}", e))?;
    
    // make executable on unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&ytdlp_path, perms)
            .map_err(|e| format!("failed to set permissions: {}", e))?;
    }
    
    log::info!("yt-dlp downloaded to: {}", ytdlp_path.display());
    Ok(())
}

// 4b. get download url based on platform
fn get_ytdlp_download_info() -> (&'static str, &'static str) {
    if cfg!(target_os = "windows") {
        (
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
            "yt-dlp.exe"
        )
    } else if cfg!(target_os = "macos") {
        (
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos",
            "yt-dlp"
        )
    } else {
        // linux and others
        (
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp",
            "yt-dlp"
        )
    }
}

// 5a. get the yt-dlp command to use
// returns the path to use when spawning yt-dlp
pub async fn get_ytdlp_command() -> Result<String, String> {
    let path = ensure_ytdlp().await?;
    Ok(path.to_string_lossy().to_string())
}

