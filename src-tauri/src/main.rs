// 1a. main.rs - entry point for godz
// 1b. sets up tauri and registers all the commands frontend can call
// 1c. pretty straightforward tbh, tauri handles most of the hard stuff

// this hides the console window on windows release builds
// cuz nobody wants a random terminal popping up
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod downloader;
mod processor;
mod scrambler;
mod jobs;
mod sfx;
mod utils;
mod setup;

use jobs::{JobManager, JobStatus, JobConfig, OutputFormat, BrollSource, OverlayPosition};
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::State;

// 2a. shared app state
// job_manager handles all the video processing jobs
// wrapped in Arc<Mutex> so multiple threads can access it
struct AppState {
    job_manager: Arc<Mutex<JobManager>>,
}

// ============================================
// TAURI COMMANDS - frontend calls these
// ============================================

// 3a. start a new job - the main function users care about
// takes all the settings and kicks off processing
#[tauri::command]
async fn start_job(
    state: State<'_, AppState>,
    youtube_links: Vec<String>,
    local_broll_paths: Option<Vec<String>>,
    user_video_path: String,
    output_format: String,
    overlay_position: Option<String>,
    custom_width: Option<u32>,
    custom_height: Option<u32>,
    split_ratio: Option<f64>,
    pip_scale: Option<f64>,
    sfx_folder: Option<String>,
) -> Result<String, String> {
    // parse output format from string
    let format = match output_format.to_lowercase().as_str() {
        "youtube" => OutputFormat::YouTube,
        "tiktok" => OutputFormat::TikTok,
        "instagram" => OutputFormat::Instagram,
        "custom" => OutputFormat::Custom,
        _ => OutputFormat::YouTube,
    };

    // parse overlay position
    let position = match overlay_position.as_deref() {
        Some("top") => OverlayPosition::Top,
        Some("bottom") => OverlayPosition::Bottom,
        Some("topleft") | Some("top-left") => OverlayPosition::TopLeft,
        Some("topright") | Some("top-right") => OverlayPosition::TopRight,
        Some("bottomleft") | Some("bottom-left") => OverlayPosition::BottomLeft,
        Some("bottomright") | Some("bottom-right") => OverlayPosition::BottomRight,
        Some("sidebyside") | Some("side-by-side") => OverlayPosition::SideBySide,
        _ => OverlayPosition::Top,
    };

    // figure out broll source - local files take priority
    let broll_source = if let Some(paths) = local_broll_paths {
        if !paths.is_empty() {
            BrollSource::LocalFiles(paths)
        } else if !youtube_links.is_empty() {
            BrollSource::YouTube(youtube_links)
        } else {
            return Err("yo you need to provide some broll - either youtube links or local files".to_string());
        }
    } else if !youtube_links.is_empty() {
        BrollSource::YouTube(youtube_links)
    } else {
        return Err("yo you need to provide some broll - either youtube links or local files".to_string());
    };

    let config = JobConfig {
        broll_source,
        user_video_path,
        output_format: format,
        sfx_folder,
        max_clip_duration: 4.0,
        min_clip_duration: 1.5,
        overlay_position: position,
        custom_width,
        custom_height,
        split_ratio: split_ratio.unwrap_or(0.5),
        pip_scale: pip_scale.unwrap_or(0.3),
    };

    // create the job and get its id
    let job_id = {
        let mut manager = state.job_manager.lock();
        manager.create_job(config)
    };

    // spawn the job to run in background
    let manager = Arc::clone(&state.job_manager);
    let id_clone = job_id.clone();
    tokio::spawn(async move {
        if let Err(e) = jobs::run_job(manager, id_clone.clone()).await {
            log::error!("job {} failed: {}", id_clone, e);
        }
    });

    Ok(job_id)
}

// 4a. get status of a specific job
// frontend polls this to update the ui
#[tauri::command]
fn get_job_status(state: State<'_, AppState>, job_id: String) -> Result<JobStatus, String> {
    let manager = state.job_manager.lock();
    manager
        .get_job_status(&job_id)
        .ok_or_else(|| format!("job {} not found", job_id))
}

// 4b. get all jobs for the sidebar
#[tauri::command]
fn get_all_jobs(state: State<'_, AppState>) -> Vec<JobStatus> {
    let manager = state.job_manager.lock();
    manager.get_all_jobs()
}

// 4c. cancel a running job
#[tauri::command]
fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let mut manager = state.job_manager.lock();
    manager.cancel_job(&job_id)
}

// 5a. check if ffmpeg and yt-dlp are installed
// we need both for the app to work properly
#[tauri::command]
fn check_dependencies() -> Result<DependencyStatus, String> {
    let ffmpeg = which::which("ffmpeg").is_ok();
    // use our setup module which checks PATH and app data dir
    let ytdlp = setup::check_ytdlp_available();
    
    // detect gpu encoder availability
    let gpu_encoder = detect_gpu_encoder_name();
    
    Ok(DependencyStatus {
        ffmpeg_installed: ffmpeg,
        ytdlp_installed: ytdlp,
        all_good: ffmpeg,  // only ffmpeg is truly required, yt-dlp can be skipped with local mode
        gpu_encoder,
    })
}

// 5b. download yt-dlp if not already available
// called from frontend when user wants to enable youtube mode
#[tauri::command]
async fn download_ytdlp() -> Result<String, String> {
    let path = setup::ensure_ytdlp().await?;
    Ok(path.to_string_lossy().to_string())
}

#[derive(serde::Serialize)]
struct DependencyStatus {
    ffmpeg_installed: bool,
    ytdlp_installed: bool,
    all_good: bool,
    gpu_encoder: String,
}

// 5b. validate youtube url before we try downloading
// catches typos early so users dont wait for nothing
#[tauri::command]
fn validate_youtube_url(url: String) -> bool {
    let patterns = [
        r"youtube\.com/watch\?v=",
        r"youtu\.be/",
        r"youtube\.com/shorts/",
    ];
    
    patterns.iter().any(|p| {
        regex::Regex::new(p)
            .map(|re| re.is_match(&url))
            .unwrap_or(false)
    })
}

// 6a. get app directories for temp files and exports
#[tauri::command]
fn get_app_dirs(app: tauri::AppHandle) -> Result<AppDirs, String> {
    let data_dir = app.path_resolver()
        .app_data_dir()
        .ok_or("couldnt get app data dir")?;
    
    let temp_dir = data_dir.join("temp");
    let exports_dir = data_dir.join("exports");
    
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&exports_dir).map_err(|e| e.to_string())?;
    
    Ok(AppDirs {
        temp: temp_dir.to_string_lossy().to_string(),
        exports: exports_dir.to_string_lossy().to_string(),
    })
}

#[derive(serde::Serialize)]
struct AppDirs {
    temp: String,
    exports: String,
}

// 7a. get available overlay positions for ui dropdown
#[tauri::command]
fn get_overlay_positions() -> Vec<OverlayOption> {
    vec![
        OverlayOption { value: "top".to_string(), label: "B-Roll on Top".to_string(), description: "Classic split - B-Roll above, you below".to_string() },
        OverlayOption { value: "bottom".to_string(), label: "B-Roll on Bottom".to_string(), description: "Split - you above, B-Roll below".to_string() },
        OverlayOption { value: "top-left".to_string(), label: "Picture in Picture (Top Left)".to_string(), description: "Small B-Roll overlay in top left corner".to_string() },
        OverlayOption { value: "top-right".to_string(), label: "Picture in Picture (Top Right)".to_string(), description: "Small B-Roll overlay in top right corner".to_string() },
        OverlayOption { value: "bottom-left".to_string(), label: "Picture in Picture (Bottom Left)".to_string(), description: "Small B-Roll overlay in bottom left corner".to_string() },
        OverlayOption { value: "bottom-right".to_string(), label: "Picture in Picture (Bottom Right)".to_string(), description: "Small B-Roll overlay in bottom right corner".to_string() },
        OverlayOption { value: "side-by-side".to_string(), label: "Side by Side".to_string(), description: "B-Roll on left, you on right".to_string() },
    ]
}

#[derive(serde::Serialize)]
struct OverlayOption {
    value: String,
    label: String,
    description: String,
}

// 7b. get output format presets
#[tauri::command]
fn get_output_formats() -> Vec<FormatOption> {
    vec![
        FormatOption { value: "youtube".to_string(), label: "YouTube".to_string(), width: 1920, height: 1080, description: "16:9 landscape for YouTube".to_string() },
        FormatOption { value: "tiktok".to_string(), label: "TikTok".to_string(), width: 1080, height: 1920, description: "9:16 portrait for TikTok/Reels".to_string() },
        FormatOption { value: "instagram".to_string(), label: "Instagram".to_string(), width: 1080, height: 1350, description: "4:5 for Instagram feed".to_string() },
        FormatOption { value: "custom".to_string(), label: "Custom".to_string(), width: 0, height: 0, description: "Pick your own dimensions".to_string() },
    ]
}

#[derive(serde::Serialize)]
struct FormatOption {
    value: String,
    label: String,
    width: u32,
    height: u32,
    description: String,
}

// helper to get gpu encoder name for status display
fn detect_gpu_encoder_name() -> String {
    if check_encoder("h264_nvenc") {
        "NVIDIA NVENC (GPU)".to_string()
    } else if check_encoder("h264_vaapi") {
        "AMD/Intel VAAPI (GPU)".to_string()
    } else if check_encoder("h264_videotoolbox") {
        "Apple VideoToolbox (GPU)".to_string()
    } else {
        "CPU (libx264)".to_string()
    }
}

fn check_encoder(encoder: &str) -> bool {
    std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(encoder)
        })
        .unwrap_or(false)
}

// ============================================
// MAIN - where we start the app
// ============================================

fn main() {
    // init logging for debugging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    log::info!("starting godz...");

    // create shared state
    let state = AppState {
        job_manager: Arc::new(Mutex::new(JobManager::new())),
    };

    // build and run tauri app
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            start_job,
            get_job_status,
            get_all_jobs,
            cancel_job,
            check_dependencies,
            download_ytdlp,
            validate_youtube_url,
            get_app_dirs,
            get_overlay_positions,
            get_output_formats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
