// 1a. processor.rs - where all the ffmpeg magic happens
// 1b. handles video cutting, scaling, compositing, the whole deal
// 1c. ngl this was the hardest part to figure out
// those ffmpeg filter chains are wild

use std::path::{Path, PathBuf};
use tokio::process::Command;
use thiserror::Error;
use crate::jobs::OverlayPosition;

// 2a. errors that can happen during processing
// using thiserror cuz writing error boilerplate sucks
#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("ffmpeg not found - install it: sudo apt install ffmpeg")]
    FfmpegNotFound,
    
    #[error("ffprobe not found - comes with ffmpeg usually")]
    FfprobeNotFound,
    
    #[error("processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("invalid video file: {0}")]
    InvalidVideo(String),
    
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type ProcessResult<T> = Result<T, ProcessorError>;

// 3a. video dimensions
// keeping it simple - just width and height
#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn youtube() -> Self {
        Self { width: 1920, height: 1080 }
    }
    
    pub fn tiktok() -> Self {
        Self { width: 1080, height: 1920 }
    }
    
    pub fn instagram() -> Self {
        Self { width: 1080, height: 1350 }
    }
}

// 3b. info we extract from videos
#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

// 3c. gpu encoder types we can use
// check which ones are available on the system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuEncoder {
    Nvenc,      // nvidia gpus - fastest usually
    Vaapi,      // amd and intel gpus on linux
    Videotoolbox, // macos hardware encoding
    None,       // fallback to cpu (libx264)
}

// 4a. the main processor struct
pub struct Processor {
    temp_dir: PathBuf,
    gpu_encoder: GpuEncoder,
}

impl Processor {
    pub fn new(temp_dir: impl AsRef<Path>) -> ProcessResult<Self> {
        let temp_dir = temp_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&temp_dir)?;
        
        // detect what gpu encoder we can use
        // this runs once when processor is created
        let gpu_encoder = detect_gpu_encoder();
        log::info!("using encoder: {:?}", gpu_encoder);
        
        Ok(Self { temp_dir, gpu_encoder })
    }

    // 5a. check if ffmpeg is available
    pub fn check_ffmpeg_installed() -> bool {
        which::which("ffmpeg").is_ok() && which::which("ffprobe").is_ok()
    }

    // 5b. get encoding args based on gpu availability
    // gpu encoding is way faster when available
    fn get_encoder_args(&self) -> Vec<String> {
        match self.gpu_encoder {
            GpuEncoder::Nvenc => vec![
                "-c:v".to_string(), "h264_nvenc".to_string(),
                "-preset".to_string(), "p4".to_string(),  // balanced preset
                "-rc".to_string(), "vbr".to_string(),
                "-cq".to_string(), "23".to_string(),
            ],
            GpuEncoder::Vaapi => vec![
                "-vaapi_device".to_string(), "/dev/dri/renderD128".to_string(),
                "-c:v".to_string(), "h264_vaapi".to_string(),
                "-qp".to_string(), "23".to_string(),
            ],
            GpuEncoder::Videotoolbox => vec![
                "-c:v".to_string(), "h264_videotoolbox".to_string(),
                "-q:v".to_string(), "65".to_string(),
            ],
            GpuEncoder::None => vec![
                "-c:v".to_string(), "libx264".to_string(),
                "-preset".to_string(), "fast".to_string(),
                "-crf".to_string(), "23".to_string(),
            ],
        }
    }

    // 6a. get video metadata using ffprobe
    // we need duration to know how many clips to cut
    pub async fn get_metadata(&self, video_path: &Path) -> ProcessResult<VideoMetadata> {
        let output = Command::new("ffprobe")
            .args([
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
                video_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(ProcessorError::InvalidVideo(
                video_path.to_string_lossy().to_string()
            ));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ProcessorError::ProcessingFailed(e.to_string()))?;

        // find the video stream in the json
        let streams = json["streams"].as_array()
            .ok_or_else(|| ProcessorError::InvalidVideo("no streams found".to_string()))?;
        
        let video_stream = streams.iter()
            .find(|s| s["codec_type"] == "video")
            .ok_or_else(|| ProcessorError::InvalidVideo("no video stream".to_string()))?;

        // parse fps from fraction format like "30000/1001"
        // why does ffmpeg make this so complicated smh
        let fps_str = video_stream["r_frame_rate"].as_str().unwrap_or("30/1");
        let fps = parse_fps(fps_str);

        let duration = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        Ok(VideoMetadata {
            duration,
            width: video_stream["width"].as_u64().unwrap_or(1920) as u32,
            height: video_stream["height"].as_u64().unwrap_or(1080) as u32,
            fps,
        })
    }

    // 7a. cut a clip from a video
    // start_time and duration in seconds
    // mute_audio is crucial for avoiding content id
    pub async fn cut_clip(
        &self,
        input_path: &Path,
        output_path: &Path,
        start_time: f64,
        duration: f64,
        mute_audio: bool,
    ) -> ProcessResult<PathBuf> {
        let mut args = vec![
            "-y".to_string(),  // overwrite output
            "-ss".to_string(), format!("{:.3}", start_time),  // seek before -i is faster
            "-i".to_string(), input_path.to_str().unwrap().to_string(),
            "-t".to_string(), format!("{:.3}", duration),
        ];

        // add encoder args
        args.extend(self.get_encoder_args());

        if mute_audio {
            args.push("-an".to_string());  // strip audio completely
        } else {
            args.extend(["-c:a".to_string(), "aac".to_string()]);
        }

        args.push(output_path.to_str().unwrap().to_string());

        let output = Command::new("ffmpeg")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("ffmpeg cut failed: {}", stderr);
            return Err(ProcessorError::ProcessingFailed(stderr.to_string()));
        }

        Ok(output_path.to_path_buf())
    }

    // 7b. scale video to target dimensions
    // maintains aspect ratio and pads with black bars
    pub async fn scale_video(
        &self,
        input_path: &Path,
        output_path: &Path,
        target: Dimensions,
    ) -> ProcessResult<PathBuf> {
        // scale filter that keeps aspect ratio and adds padding
        let filter = format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black",
            target.width, target.height, target.width, target.height
        );

        let mut args = vec![
            "-y".to_string(),
            "-i".to_string(), input_path.to_str().unwrap().to_string(),
            "-vf".to_string(), filter,
        ];
        
        args.extend(self.get_encoder_args());
        args.extend([
            "-c:a".to_string(), "copy".to_string(),
            output_path.to_str().unwrap().to_string(),
        ]);

        let output = Command::new("ffmpeg")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessorError::ProcessingFailed(stderr.to_string()));
        }

        Ok(output_path.to_path_buf())
    }

    // 8a. composite split screen - broll top, user bottom (or vice versa)
    // split_ratio determines how much screen broll takes
    pub async fn composite_split_screen(
        &self,
        top_video: &Path,
        bottom_video: &Path,
        output_path: &Path,
        target: Dimensions,
        split_ratio: f64,
    ) -> ProcessResult<PathBuf> {
        // calculate heights for each section
        let top_height = (target.height as f64 * split_ratio) as u32;
        let bottom_height = target.height - top_height;

        // complex filter to scale and stack videos
        // [0] is top video, [1] is bottom video
        let filter = format!(
            "[0:v]scale={}:{},setsar=1[top];\
             [1:v]scale={}:{},setsar=1[bottom];\
             [top][bottom]vstack=inputs=2[out]",
            target.width, top_height,
            target.width, bottom_height
        );

        let mut args = vec![
            "-y".to_string(),
            "-i".to_string(), top_video.to_str().unwrap().to_string(),
            "-i".to_string(), bottom_video.to_str().unwrap().to_string(),
            "-filter_complex".to_string(), filter,
            "-map".to_string(), "[out]".to_string(),
            "-map".to_string(), "1:a?".to_string(),  // audio from bottom (user) video
        ];
        
        args.extend(self.get_encoder_args());
        args.extend([
            "-c:a".to_string(), "aac".to_string(),
            "-b:a".to_string(), "192k".to_string(),
            output_path.to_str().unwrap().to_string(),
        ]);

        let output = Command::new("ffmpeg")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("composite failed: {}", stderr);
            return Err(ProcessorError::ProcessingFailed(stderr.to_string()));
        }

        Ok(output_path.to_path_buf())
    }

    // 8b. picture in picture composite
    // main video fills screen, overlay is small in corner
    pub async fn composite_pip(
        &self,
        main_video: &Path,
        overlay_video: &Path,
        output_path: &Path,
        target: Dimensions,
        position: OverlayPosition,
        pip_scale: f64,  // 0.0-1.0, how much of screen pip takes
    ) -> ProcessResult<PathBuf> {
        let pip_width = (target.width as f64 * pip_scale) as u32;
        let pip_height = (target.height as f64 * pip_scale) as u32;
        let padding = 20;  // pixels from edge

        // calculate pip position based on overlay setting
        let (x_pos, y_pos) = match position {
            OverlayPosition::TopLeft => (padding, padding),
            OverlayPosition::TopRight => (target.width as i32 - pip_width as i32 - padding, padding),
            OverlayPosition::BottomLeft => (padding, target.height as i32 - pip_height as i32 - padding),
            OverlayPosition::BottomRight => (
                target.width as i32 - pip_width as i32 - padding,
                target.height as i32 - pip_height as i32 - padding
            ),
            _ => (padding, padding),  // default to top left
        };

        // filter chain:
        // 1. scale main to target dimensions
        // 2. scale overlay to pip size
        // 3. overlay pip on main
        let filter = format!(
            "[0:v]scale={}:{},setsar=1[main];\
             [1:v]scale={}:{},setsar=1[pip];\
             [main][pip]overlay={}:{}[out]",
            target.width, target.height,
            pip_width, pip_height,
            x_pos, y_pos
        );

        let mut args = vec![
            "-y".to_string(),
            "-i".to_string(), main_video.to_str().unwrap().to_string(),
            "-i".to_string(), overlay_video.to_str().unwrap().to_string(),
            "-filter_complex".to_string(), filter,
            "-map".to_string(), "[out]".to_string(),
            "-map".to_string(), "0:a?".to_string(),  // audio from main video
        ];
        
        args.extend(self.get_encoder_args());
        args.extend([
            "-c:a".to_string(), "aac".to_string(),
            "-b:a".to_string(), "192k".to_string(),
            output_path.to_str().unwrap().to_string(),
        ]);

        let output = Command::new("ffmpeg")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("pip composite failed: {}", stderr);
            return Err(ProcessorError::ProcessingFailed(stderr.to_string()));
        }

        Ok(output_path.to_path_buf())
    }

    // 8c. side by side composite
    // broll on left, user on right (or based on ratio)
    pub async fn composite_side_by_side(
        &self,
        left_video: &Path,
        right_video: &Path,
        output_path: &Path,
        target: Dimensions,
        left_ratio: f64,  // how much screen left video takes
    ) -> ProcessResult<PathBuf> {
        let left_width = (target.width as f64 * left_ratio) as u32;
        let right_width = target.width - left_width;

        // scale both videos and stack horizontally
        let filter = format!(
            "[0:v]scale={}:{},setsar=1[left];\
             [1:v]scale={}:{},setsar=1[right];\
             [left][right]hstack=inputs=2[out]",
            left_width, target.height,
            right_width, target.height
        );

        let mut args = vec![
            "-y".to_string(),
            "-i".to_string(), left_video.to_str().unwrap().to_string(),
            "-i".to_string(), right_video.to_str().unwrap().to_string(),
            "-filter_complex".to_string(), filter,
            "-map".to_string(), "[out]".to_string(),
            "-map".to_string(), "1:a?".to_string(),  // audio from right (user) video
        ];
        
        args.extend(self.get_encoder_args());
        args.extend([
            "-c:a".to_string(), "aac".to_string(),
            "-b:a".to_string(), "192k".to_string(),
            output_path.to_str().unwrap().to_string(),
        ]);

        let output = Command::new("ffmpeg")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("side by side composite failed: {}", stderr);
            return Err(ProcessorError::ProcessingFailed(stderr.to_string()));
        }

        Ok(output_path.to_path_buf())
    }

    // 9a. concatenate multiple clips into one video
    // uses ffmpeg concat demuxer - fast cuz it just copies streams
    pub async fn concat_clips(
        &self,
        clip_paths: &[PathBuf],
        output_path: &Path,
    ) -> ProcessResult<PathBuf> {
        if clip_paths.is_empty() {
            return Err(ProcessorError::ProcessingFailed("no clips to concat bruh".to_string()));
        }

        // create temp file listing all clips
        // concat demuxer needs this specific format
        let list_path = self.temp_dir.join("concat_list.txt");
        let list_content: String = clip_paths
            .iter()
            .map(|p| format!("file '{}'\n", p.to_str().unwrap()))
            .collect();
        
        std::fs::write(&list_path, list_content)?;

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-f", "concat",
                "-safe", "0",  // allow absolute paths
                "-i", list_path.to_str().unwrap(),
                "-c", "copy",  // just copy streams, no re-encode
                output_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        // cleanup temp file
        let _ = std::fs::remove_file(&list_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessorError::ProcessingFailed(stderr.to_string()));
        }

        Ok(output_path.to_path_buf())
    }

    // 10a. add sound effects at specific timestamps
    // sfx_events is (timestamp_seconds, path_to_sfx_file)
    pub async fn add_sfx(
        &self,
        video_path: &Path,
        sfx_events: &[(f64, PathBuf)],
        output_path: &Path,
    ) -> ProcessResult<PathBuf> {
        if sfx_events.is_empty() {
            // no sfx to add, just copy the file
            std::fs::copy(video_path, output_path)?;
            return Ok(output_path.to_path_buf());
        }

        // build complex audio filter
        // delay each sfx to its timestamp then mix together
        let mut inputs = vec!["-i".to_string(), video_path.to_str().unwrap().to_string()];
        let mut filter_parts = Vec::new();
        
        for (i, (timestamp, sfx_path)) in sfx_events.iter().enumerate() {
            inputs.push("-i".to_string());
            inputs.push(sfx_path.to_str().unwrap().to_string());
            
            let delay_ms = (timestamp * 1000.0) as u64;
            filter_parts.push(format!(
                "[{}:a]adelay={}|{}[sfx{}]",
                i + 1, delay_ms, delay_ms, i
            ));
        }
        
        // mix all sfx together
        let sfx_inputs: String = (0..sfx_events.len())
            .map(|i| format!("[sfx{}]", i))
            .collect();
        filter_parts.push(format!(
            "{}amix=inputs={}[sfxmix]",
            sfx_inputs,
            sfx_events.len()
        ));
        
        // mix sfx with original audio
        filter_parts.push("[0:a][sfxmix]amix=inputs=2:duration=first[out]".to_string());
        
        let filter = filter_parts.join(";");

        let mut args = inputs;
        args.extend([
            "-filter_complex".to_string(), filter,
            "-map".to_string(), "0:v".to_string(),
            "-map".to_string(), "[out]".to_string(),
            "-c:v".to_string(), "copy".to_string(),
            "-c:a".to_string(), "aac".to_string(),
            "-y".to_string(),
            output_path.to_str().unwrap().to_string(),
        ]);

        let output = Command::new("ffmpeg")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessorError::ProcessingFailed(stderr.to_string()));
        }

        Ok(output_path.to_path_buf())
    }
}

// helper to parse fps from ffprobes fraction format
// comes as "30000/1001" for 29.97fps cuz why not i guess
fn parse_fps(fps_str: &str) -> f64 {
    if let Some((num, den)) = fps_str.split_once('/') {
        let numerator: f64 = num.parse().unwrap_or(30.0);
        let denominator: f64 = den.parse().unwrap_or(1.0);
        if denominator > 0.0 {
            return numerator / denominator;
        }
    }
    30.0  // fallback to 30fps
}

// detect what gpu encoder is available
// checks in order of preference: nvidia > vaapi > videotoolbox > cpu
fn detect_gpu_encoder() -> GpuEncoder {
    // check for nvidia encoder
    // nvenc is usually the fastest option
    if check_encoder_available("h264_nvenc") {
        return GpuEncoder::Nvenc;
    }
    
    // check for vaapi (amd/intel on linux)
    if check_encoder_available("h264_vaapi") {
        return GpuEncoder::Vaapi;
    }
    
    // check for videotoolbox (macos)
    if check_encoder_available("h264_videotoolbox") {
        return GpuEncoder::Videotoolbox;
    }
    
    // no gpu encoder found, use cpu
    GpuEncoder::None
}

// check if a specific encoder is available in ffmpeg
fn check_encoder_available(encoder: &str) -> bool {
    std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(encoder)
        })
        .unwrap_or(false)
}
