// 1a. scrambler.rs - this is the secret sauce
// 1b. chops up videos into clips and mixes em together
// 1c. the whole point is to make "original" content from broll
// took forever to figure out the right clip lengths tbh

use std::path::{Path, PathBuf};
use rand::seq::SliceRandom;
use rand::Rng;
use crate::processor::Processor;
use crate::downloader::VideoInfo;

// 2a. config for how we scramble clips
// 2b. these values are tuned based on content id research
// 2c. keeping clips under 4 sec is key - most thresholds are 7-10
#[derive(Debug, Clone)]
pub struct ScrambleConfig {
    pub max_clip_duration: f64,  // longest clip allowed
    pub min_clip_duration: f64,  // shortest clip allowed
    pub duration_variance: f64,  // randomness in length (0-1)
    pub randomize_order: bool,   // shuffle clips from diff sources
}

impl Default for ScrambleConfig {
    fn default() -> Self {
        Self {
            max_clip_duration: 4.0,  // sweet spot
            min_clip_duration: 1.5,  // shorter looks choppy
            duration_variance: 0.5,
            randomize_order: true,   // always shuffle for uniqueness
        }
    }
}

// 3a. describes a clip we plan to cut
// hasnt been cut yet, just the spec
#[derive(Debug, Clone)]
pub struct ClipSpec {
    pub source_idx: usize,     // which source video
    pub start_time: f64,       // where to start (seconds)
    pub duration: f64,         // how long (seconds)
}

// 3b. a clip that actually exists on disk
#[derive(Debug, Clone)]
pub struct CutClip {
    pub path: PathBuf,
    pub source_url: String,    // keep track for attribution
    pub duration: f64,
}

// 4a. the scrambler - does the clip magic
pub struct Scrambler {
    config: ScrambleConfig,
    processor: Processor,
    temp_dir: PathBuf,
}

impl Scrambler {
    pub fn new(config: ScrambleConfig, temp_dir: impl AsRef<Path>) -> Result<Self, String> {
        let temp_dir = temp_dir.as_ref().to_path_buf();
        let processor = Processor::new(&temp_dir)
            .map_err(|e| e.to_string())?;
        
        Ok(Self {
            config,
            processor,
            temp_dir,
        })
    }

    // 5a. plan_clips - figures out where to cut
    // 5b. takes total duration we need (user video length)
    // 5c. returns list of clip specs
    // ngl this algorithm took a lot of trial and error
    pub fn plan_clips(&self, sources: &[VideoInfo], target_duration: f64) -> Vec<ClipSpec> {
        let mut rng = rand::thread_rng();
        let mut clips = Vec::new();
        let mut total_duration = 0.0;
        
        // collect all possible clip start positions
        let mut all_positions: Vec<(usize, f64)> = Vec::new();
        
        for (idx, source) in sources.iter().enumerate() {
            // skip videos shorter than min clip
            if source.duration < self.config.min_clip_duration {
                log::warn!("skipping {} - too short ({:.1}s)", source.title, source.duration);
                continue;
            }
            
            // generate positions throughout the video
            let mut pos = 0.0;
            while pos + self.config.min_clip_duration <= source.duration {
                all_positions.push((idx, pos));
                // random step for variety
                pos += rng.gen_range(1.0..3.0);
            }
        }
        
        if all_positions.is_empty() {
            log::error!("no valid positions to cut from!");
            return clips;
        }
        
        // shuffle for randomness
        if self.config.randomize_order {
            all_positions.shuffle(&mut rng);
        }
        
        // 6a. pick clips until we hit target duration
        // 6b. track used ranges to avoid overlap
        let mut used_ranges: Vec<Vec<(f64, f64)>> = vec![Vec::new(); sources.len()];
        let mut pos_idx = 0;
        
        while total_duration < target_duration && pos_idx < all_positions.len() {
            let (source_idx, start) = all_positions[pos_idx];
            pos_idx += 1;
            
            // random duration within config bounds
            let base = (self.config.max_clip_duration + self.config.min_clip_duration) / 2.0;
            let variance = (self.config.max_clip_duration - self.config.min_clip_duration) 
                * self.config.duration_variance;
            let duration = base + rng.gen_range(-variance..variance);
            let duration = duration.clamp(self.config.min_clip_duration, self.config.max_clip_duration);
            
            // dont go past end of source
            let source_dur = sources[source_idx].duration;
            let actual_dur = duration.min(source_dur - start);
            
            if actual_dur < self.config.min_clip_duration {
                continue;
            }
            
            // check for overlap with used ranges
            let end = start + actual_dur;
            let overlaps = used_ranges[source_idx].iter().any(|(s, e)| {
                start < *e && end > *s
            });
            
            if overlaps {
                continue;
            }
            
            // mark range as used
            used_ranges[source_idx].push((start, end));
            
            clips.push(ClipSpec {
                source_idx,
                start_time: start,
                duration: actual_dur,
            });
            
            total_duration += actual_dur;
            
            // if we ran out of positions, shuffle and try again
            if pos_idx >= all_positions.len() && total_duration < target_duration {
                pos_idx = 0;
                if self.config.randomize_order {
                    all_positions.shuffle(&mut rng);
                }
            }
        }
        
        log::info!("planned {} clips, {:.1}s total (needed {:.1}s)", 
            clips.len(), total_duration, target_duration);
        
        clips
    }

    // 7a. cut_clips - actually cuts the videos
    // 7b. calls ffmpeg for each clip
    // 7c. mutes audio (crucial for avoiding detection)
    pub async fn cut_clips(
        &self,
        sources: &[VideoInfo],
        clip_specs: &[ClipSpec],
    ) -> Result<Vec<CutClip>, String> {
        let mut cut_clips = Vec::new();
        
        for (i, spec) in clip_specs.iter().enumerate() {
            let source = &sources[spec.source_idx];
            
            let clip_filename = format!("clip_{:04}.mp4", i);
            let clip_path = self.temp_dir.join(&clip_filename);
            
            log::info!("cutting clip {} from {} @ {:.1}s ({:.1}s)", 
                i, source.title, spec.start_time, spec.duration);
            
            // cut with audio muted - this is important!
            // audio fingerprinting catches way more than video
            self.processor.cut_clip(
                &source.path,
                &clip_path,
                spec.start_time,
                spec.duration,
                true,  // mute audio
            ).await.map_err(|e| e.to_string())?;
            
            cut_clips.push(CutClip {
                path: clip_path,
                source_url: source.source_url.clone(),
                duration: spec.duration,
            });
        }
        
        Ok(cut_clips)
    }

    // 8a. concat_clips - joins all clips into one video
    // uses ffmpeg concat demuxer which is fast
    pub async fn concat_clips(&self, clips: &[CutClip]) -> Result<PathBuf, String> {
        let output_path = self.temp_dir.join("broll_concat.mp4");
        let clip_paths: Vec<PathBuf> = clips.iter().map(|c| c.path.clone()).collect();
        
        self.processor.concat_clips(&clip_paths, &output_path)
            .await
            .map_err(|e| e.to_string())
    }
}

// ============================================
// WHY THIS WORKS - content id avoidance notes
// ============================================
// 
// based on research into how youtube/tiktok detect copyrighted content:
//
// 1. max 4 seconds per clip
//    - content owners usually set thresholds at 7-10 seconds
//    - staying well under gives us wiggle room
//
// 2. muting audio is critical
//    - audio fingerprinting is way more accurate than video
//    - even 1-2 seconds of audio can trigger detection
//    - without audio, only visual matching matters
//
// 3. rotating between sources
//    - content id tracks cumulative time from each source
//    - mixing sources means no single source hits threshold
//
// 4. randomizing order
//    - creates a unique "fingerprint" each time
//    - same clips in different order = different content
//
// 5. avoiding repeated clips
//    - we track used ranges per source
//    - no clip appears twice
//
// this isnt 100% bulletproof but it works most of the time
// the key insight is were making something genuinely new
// from the combination of sources
