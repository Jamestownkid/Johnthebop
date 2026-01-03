// 1a. jobs.rs - handles the job queue system for godz
// 1b. basically lets users stack up multiple video jobs
// 1c. runs em in the background so ui stays snappy
// tbh this took forever to get right with the async stuff

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::downloader::{Downloader, VideoInfo};
use crate::processor::{Processor, Dimensions};
use crate::scrambler::{Scrambler, ScrambleConfig};

// 2a. output format presets for different platforms
// each platform has their own aspect ratio preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    YouTube,    // 1920x1080 - landscape boi
    TikTok,     // 1080x1920 - vertical gang
    Instagram,  // 1080x1350 - 4:5 ratio
    Custom,     // user picks their own dimensions
}

// 2b. where the broll footage comes from
// youtube mode downloads, local mode uses files on disk
#[derive(Debug, Clone)]
pub enum BrollSource {
    YouTube(Vec<String>),      // list of yt URLs
    LocalFiles(Vec<String>),   // paths to local files
}

// 2c. overlay position - where broll shows up on screen
// this is the new feature users asked for
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OverlayPosition {
    Top,         // broll on top, user on bottom (classic)
    Bottom,      // broll on bottom, user on top
    TopLeft,     // broll as pip in top left corner
    TopRight,    // broll as pip in top right corner
    BottomLeft,  // broll as pip in bottom left
    BottomRight, // broll as pip in bottom right
    SideBySide,  // broll on left, user on right
}

impl Default for OverlayPosition {
    fn default() -> Self {
        OverlayPosition::Top
    }
}

impl OutputFormat {
    pub fn dimensions(&self) -> Dimensions {
        match self {
            OutputFormat::YouTube => Dimensions::youtube(),
            OutputFormat::TikTok => Dimensions::tiktok(),
            OutputFormat::Instagram => Dimensions::instagram(),
            OutputFormat::Custom => Dimensions::youtube(), // fallback, actual dims come from config
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            OutputFormat::YouTube => "YouTube",
            OutputFormat::TikTok => "TikTok",
            OutputFormat::Instagram => "Instagram",
            OutputFormat::Custom => "Custom",
        }
    }
}

// 3a. config for a single job - all the settings user picks
#[derive(Debug, Clone)]
pub struct JobConfig {
    pub broll_source: BrollSource,     // where broll comes from
    pub user_video_path: String,        // the talking head video
    pub output_format: OutputFormat,    // preset or custom
    pub sfx_folder: Option<String>,     // optional sound effects
    pub max_clip_duration: f64,         // longest a clip can be (default 4s)
    pub min_clip_duration: f64,         // shortest a clip can be (default 1.5s)
    pub overlay_position: OverlayPosition,  // where broll shows
    pub custom_width: Option<u32>,      // for custom output size
    pub custom_height: Option<u32>,     // for custom output size
    pub split_ratio: f64,               // how much screen broll takes (0.3-0.7)
    pub pip_scale: f64,                 // for pip modes, how big the overlay is
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            broll_source: BrollSource::LocalFiles(vec![]),
            user_video_path: String::new(),
            output_format: OutputFormat::YouTube,
            sfx_folder: None,
            max_clip_duration: 4.0,
            min_clip_duration: 1.5,
            overlay_position: OverlayPosition::Top,
            custom_width: None,
            custom_height: None,
            split_ratio: 0.5,
            pip_scale: 0.3, // 30% of screen for pip
        }
    }
}

// 3b. states a job can be in
// pretty self explanatory tbh
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobState {
    Queued,       // waiting to start
    Downloading,  // getting broll from youtube
    Processing,   // cutting and scrambling clips
    Compositing,  // combining broll with user video
    Finalizing,   // adding sfx, final touches
    Complete,     // done! output ready
    Failed,       // something broke :(
    Cancelled,    // user said nvm
}

// 3c. progress info for the ui to display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub stage: String,              // human readable stage name
    pub percent: f32,               // 0-100 progress
    pub current_item: Option<String>,    // what were working on rn
    pub total_items: Option<usize>,      // how many things total
    pub completed_items: Option<usize>,  // how many done so far
}

// 3d. full status object sent to frontend
// this is what the UI polls for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub id: String,
    pub state: JobState,
    pub progress: JobProgress,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub output_format: String,
    pub overlay_position: String,
}

// 4a. internal job struct - not serialized to frontend
// has more detailed info needed for processing
struct Job {
    id: String,
    config: JobConfig,
    state: JobState,
    progress: JobProgress,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    output_path: Option<PathBuf>,
    error: Option<String>,
    cancelled: bool,
}

impl Job {
    fn new(id: String, config: JobConfig) -> Self {
        Self {
            id,
            config,
            state: JobState::Queued,
            progress: JobProgress {
                stage: "Queued - waiting to start".to_string(),
                percent: 0.0,
                current_item: None,
                total_items: None,
                completed_items: None,
            },
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            output_path: None,
            error: None,
            cancelled: false,
        }
    }
    
    // convert to status for frontend
    fn to_status(&self) -> JobStatus {
        let overlay_name = match self.config.overlay_position {
            OverlayPosition::Top => "Top",
            OverlayPosition::Bottom => "Bottom",
            OverlayPosition::TopLeft => "Top Left",
            OverlayPosition::TopRight => "Top Right",
            OverlayPosition::BottomLeft => "Bottom Left",
            OverlayPosition::BottomRight => "Bottom Right",
            OverlayPosition::SideBySide => "Side by Side",
        };
        
        JobStatus {
            id: self.id.clone(),
            state: self.state.clone(),
            progress: self.progress.clone(),
            created_at: self.created_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            output_path: self.output_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            error: self.error.clone(),
            output_format: self.config.output_format.name().to_string(),
            overlay_position: overlay_name.to_string(),
        }
    }
}

// 5a. job manager - stores and manages all jobs
// the brain of the operation basically
pub struct JobManager {
    jobs: HashMap<String, Job>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    // 5b. create a new job
    // generates a short id for easier reference
    pub fn create_job(&mut self, config: JobConfig) -> String {
        // use first 8 chars of uuid - unique enough and easier to read
        let id = Uuid::new_v4().to_string()[..8].to_string();
        let job = Job::new(id.clone(), config);
        self.jobs.insert(id.clone(), job);
        log::info!("created job: {}", id);
        id
    }

    // 5c. get status of a job by id
    pub fn get_job_status(&self, id: &str) -> Option<JobStatus> {
        self.jobs.get(id).map(|j| j.to_status())
    }

    // 5d. get all jobs sorted by creation time (newest first)
    pub fn get_all_jobs(&self) -> Vec<JobStatus> {
        let mut jobs: Vec<_> = self.jobs.values().map(|j| j.to_status()).collect();
        jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        jobs
    }

    // 5e. cancel a job thats running
    // cant cancel if already done or failed
    pub fn cancel_job(&mut self, id: &str) -> Result<(), String> {
        if let Some(job) = self.jobs.get_mut(id) {
            if job.state == JobState::Complete || job.state == JobState::Failed {
                return Err("cant cancel a finished job homie".to_string());
            }
            job.cancelled = true;
            job.state = JobState::Cancelled;
            log::info!("cancelled job: {}", id);
            Ok(())
        } else {
            Err(format!("job {} not found", id))
        }
    }

    // internal update methods - not pub cuz only used by run_job
    fn update_state(&mut self, id: &str, state: JobState) {
        if let Some(job) = self.jobs.get_mut(id) {
            job.state = state.clone();
            if state == JobState::Downloading && job.started_at.is_none() {
                job.started_at = Some(Utc::now());
            }
        }
    }

    fn update_progress(&mut self, id: &str, progress: JobProgress) {
        if let Some(job) = self.jobs.get_mut(id) {
            job.progress = progress;
        }
    }

    fn set_complete(&mut self, id: &str, output_path: PathBuf) {
        if let Some(job) = self.jobs.get_mut(id) {
            job.state = JobState::Complete;
            job.output_path = Some(output_path);
            job.completed_at = Some(Utc::now());
            job.progress = JobProgress {
                stage: "All done! Your video is ready".to_string(),
                percent: 100.0,
                current_item: None,
                total_items: None,
                completed_items: None,
            };
        }
    }

    fn set_failed(&mut self, id: &str, error: String) {
        if let Some(job) = self.jobs.get_mut(id) {
            job.state = JobState::Failed;
            job.error = Some(error.clone());
            job.completed_at = Some(Utc::now());
            job.progress.stage = format!("Failed: {}", error);
        }
    }

    fn is_cancelled(&self, id: &str) -> bool {
        self.jobs.get(id).map(|j| j.cancelled).unwrap_or(false)
    }

    fn get_config(&self, id: &str) -> Option<JobConfig> {
        self.jobs.get(id).map(|j| j.config.clone())
    }
}

// 6a. run_job - the main processing pipeline
// this is where all the actual work happens
// spawned in a tokio task so it runs in background
pub async fn run_job(manager: Arc<Mutex<JobManager>>, job_id: String) -> Result<(), String> {
    log::info!("starting job: {}", job_id);
    
    // grab the config
    let config = {
        let mgr = manager.lock();
        mgr.get_config(&job_id).ok_or("job not found yo")?
    };
    
    // set up temp directories for this job
    // each job gets its own folder so they dont interfere
    let temp_base = std::env::temp_dir().join("godz").join(&job_id);
    let downloads_dir = temp_base.join("downloads");
    let clips_dir = temp_base.join("clips");
    let output_dir = temp_base.join("output");
    
    std::fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&clips_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    // helper closure to check if user cancelled
    let check_cancelled = || {
        let mgr = manager.lock();
        mgr.is_cancelled(&job_id)
    };

    // ============================================
    // STAGE 1: GET BROLL VIDEOS
    // download from youtube OR load local files
    // ============================================
    {
        let mut mgr = manager.lock();
        mgr.update_state(&job_id, JobState::Downloading);
        mgr.update_progress(&job_id, JobProgress {
            stage: "Getting your B-Roll ready...".to_string(),
            percent: 0.0,
            current_item: None,
            total_items: None,
            completed_items: Some(0),
        });
    }

    let mut downloaded_videos: Vec<VideoInfo> = Vec::new();
    
    match &config.broll_source {
        BrollSource::YouTube(links) => {
            // youtube mode - download each video with yt-dlp
            log::info!("youtube mode: downloading {} videos", links.len());
            
            {
                let mut mgr = manager.lock();
                mgr.update_progress(&job_id, JobProgress {
                    stage: "Downloading B-Roll from YouTube...".to_string(),
                    percent: 0.0,
                    current_item: None,
                    total_items: Some(links.len()),
                    completed_items: Some(0),
                });
            }
            
            let downloader = Downloader::new(&downloads_dir).map_err(|e| e.to_string())?;
            
            for (i, url) in links.iter().enumerate() {
                if check_cancelled() {
                    return Err("cancelled by user".to_string());
                }
                
                {
                    let mut mgr = manager.lock();
                    mgr.update_progress(&job_id, JobProgress {
                        stage: "Downloading B-Roll from YouTube...".to_string(),
                        percent: (i as f32 / links.len() as f32) * 25.0,
                        current_item: Some(url.clone()),
                        total_items: Some(links.len()),
                        completed_items: Some(i),
                    });
                }
                
                match downloader.download_video(url).await {
                    Ok(info) => {
                        log::info!("downloaded: {} ({:.1}s)", info.title, info.duration);
                        downloaded_videos.push(info);
                    }
                    Err(e) => {
                        // dont fail the whole job if one video fails
                        // just skip it and continue
                        log::warn!("failed to download {}: {}", url, e);
                    }
                }
            }
        }
        BrollSource::LocalFiles(paths) => {
            // local mode - just use files from disk
            log::info!("local mode: using {} files", paths.len());
            
            {
                let mut mgr = manager.lock();
                mgr.update_progress(&job_id, JobProgress {
                    stage: "Loading your local B-Roll files...".to_string(),
                    percent: 10.0,
                    current_item: None,
                    total_items: Some(paths.len()),
                    completed_items: Some(0),
                });
            }
            
            let processor = Processor::new(&clips_dir).map_err(|e| e.to_string())?;
            
            for (i, path_str) in paths.iter().enumerate() {
                if check_cancelled() {
                    return Err("cancelled by user".to_string());
                }
                
                let path = PathBuf::from(path_str);
                
                match processor.get_metadata(&path).await {
                    Ok(metadata) => {
                        let filename = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| format!("video_{}", i));
                        
                        downloaded_videos.push(VideoInfo {
                            path,
                            title: filename,
                            duration: metadata.duration,
                            source_url: path_str.clone(),
                        });
                    }
                    Err(e) => {
                        log::warn!("couldnt read {}: {}", path_str, e);
                    }
                }
                
                {
                    let mut mgr = manager.lock();
                    mgr.update_progress(&job_id, JobProgress {
                        stage: "Loading your local B-Roll files...".to_string(),
                        percent: 10.0 + (i as f32 / paths.len() as f32) * 15.0,
                        current_item: Some(path_str.clone()),
                        total_items: Some(paths.len()),
                        completed_items: Some(i + 1),
                    });
                }
            }
        }
    }
    
    if downloaded_videos.is_empty() {
        let mut mgr = manager.lock();
        mgr.set_failed(&job_id, "couldnt load any broll videos".to_string());
        return Err("no videos loaded".to_string());
    }

    // ============================================
    // STAGE 2: PROCESS AND SCRAMBLE CLIPS
    // cut broll into short clips and shuffle em
    // ============================================
    if check_cancelled() {
        return Err("cancelled by user".to_string());
    }
    
    {
        let mut mgr = manager.lock();
        mgr.update_state(&job_id, JobState::Processing);
        mgr.update_progress(&job_id, JobProgress {
            stage: "Processing clips...".to_string(),
            percent: 25.0,
            current_item: None,
            total_items: None,
            completed_items: None,
        });
    }

    // get the user's video duration so we know how much broll to make
    let processor = Processor::new(&clips_dir).map_err(|e| e.to_string())?;
    let user_video_path = PathBuf::from(&config.user_video_path);
    let user_metadata = processor.get_metadata(&user_video_path)
        .await
        .map_err(|e| e.to_string())?;
    
    log::info!("user video duration: {:.1}s", user_metadata.duration);

    // plan out the clips
    let scramble_config = ScrambleConfig {
        max_clip_duration: config.max_clip_duration,
        min_clip_duration: config.min_clip_duration,
        duration_variance: 0.5,
        randomize_order: true,
    };
    
    let scrambler = Scrambler::new(scramble_config, &clips_dir).map_err(|e| e)?;
    let clip_specs = scrambler.plan_clips(&downloaded_videos, user_metadata.duration);
    
    {
        let mut mgr = manager.lock();
        mgr.update_progress(&job_id, JobProgress {
            stage: "Cutting clips...".to_string(),
            percent: 35.0,
            current_item: None,
            total_items: Some(clip_specs.len()),
            completed_items: Some(0),
        });
    }
    
    let cut_clips = scrambler.cut_clips(&downloaded_videos, &clip_specs)
        .await
        .map_err(|e| e)?;

    // concat all clips into one broll video
    {
        let mut mgr = manager.lock();
        mgr.update_progress(&job_id, JobProgress {
            stage: "Joining clips together...".to_string(),
            percent: 60.0,
            current_item: None,
            total_items: None,
            completed_items: None,
        });
    }
    
    let broll_path = scrambler.concat_clips(&cut_clips).await?;

    // ============================================
    // STAGE 3: COMPOSITE FINAL VIDEO
    // combine broll with user video based on overlay settings
    // ============================================
    if check_cancelled() {
        return Err("cancelled by user".to_string());
    }
    
    {
        let mut mgr = manager.lock();
        mgr.update_state(&job_id, JobState::Compositing);
        mgr.update_progress(&job_id, JobProgress {
            stage: "Creating your final video...".to_string(),
            percent: 75.0,
            current_item: None,
            total_items: None,
            completed_items: None,
        });
    }

    // figure out output dimensions
    let dimensions = if let OutputFormat::Custom = config.output_format {
        Dimensions {
            width: config.custom_width.unwrap_or(1920),
            height: config.custom_height.unwrap_or(1080),
        }
    } else {
        config.output_format.dimensions()
    };

    let final_output = output_dir.join(format!(
        "godz_{}_{}.mp4",
        config.output_format.name().to_lowercase(),
        &job_id
    ));
    
    // composite based on overlay position
    match config.overlay_position {
        OverlayPosition::Top => {
            processor.composite_split_screen(
                &broll_path,
                &user_video_path,
                &final_output,
                dimensions,
                config.split_ratio,
            ).await.map_err(|e| e.to_string())?;
        }
        OverlayPosition::Bottom => {
            processor.composite_split_screen(
                &user_video_path,  // swap order - user on top
                &broll_path,
                &final_output,
                dimensions,
                1.0 - config.split_ratio,
            ).await.map_err(|e| e.to_string())?;
        }
        OverlayPosition::TopLeft | OverlayPosition::TopRight |
        OverlayPosition::BottomLeft | OverlayPosition::BottomRight => {
            // picture in picture mode
            processor.composite_pip(
                &user_video_path,  // user is main video
                &broll_path,       // broll is the overlay
                &final_output,
                dimensions,
                config.overlay_position,
                config.pip_scale,
            ).await.map_err(|e| e.to_string())?;
        }
        OverlayPosition::SideBySide => {
            processor.composite_side_by_side(
                &broll_path,
                &user_video_path,
                &final_output,
                dimensions,
                config.split_ratio,
            ).await.map_err(|e| e.to_string())?;
        }
    }

    // ============================================
    // STAGE 4: FINALIZE
    // add sfx if provided and cleanup
    // ============================================
    {
        let mut mgr = manager.lock();
        mgr.update_state(&job_id, JobState::Finalizing);
        mgr.update_progress(&job_id, JobProgress {
            stage: "Adding finishing touches...".to_string(),
            percent: 95.0,
            current_item: None,
            total_items: None,
            completed_items: None,
        });
    }

    // TODO: add sfx if config.sfx_folder is set
    // would use sfx.rs module here

    // mark complete
    {
        let mut mgr = manager.lock();
        mgr.set_complete(&job_id, final_output.clone());
    }
    
    log::info!("job {} complete: {}", job_id, final_output.display());
    Ok(())
}
