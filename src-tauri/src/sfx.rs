// 1a. sfx.rs - sound effects library
// 1b. handles the 10 major sfx types
// 1c. user provides a folder, we match files to types
// this part is kinda extra but makes videos pop

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;

// 2a. the 10 sfx types we support
// named after the sound they make cuz thats how ppl name files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfxType {
    Ching,      // cash register, coin drop
    Riser,      // tension building swoosh UP
    Faller,     // whoosh going DOWN
    Whoosh,     // fast transition swoosh
    Pop,        // bubble pop, notification
    Boom,       // bass drop, impact
    Glitch,     // digital stutter
    Click,      // ui tap, button
    Sparkle,    // magic shimmer
    Thud,       // heavy impact
}

impl SfxType {
    pub fn all() -> &'static [SfxType] {
        &[
            SfxType::Ching,
            SfxType::Riser,
            SfxType::Faller,
            SfxType::Whoosh,
            SfxType::Pop,
            SfxType::Boom,
            SfxType::Glitch,
            SfxType::Click,
            SfxType::Sparkle,
            SfxType::Thud,
        ]
    }

    // 3a. match filename to sfx type
    // looks for keywords in the filename
    pub fn from_filename(filename: &str) -> Option<SfxType> {
        let lower = filename.to_lowercase();
        
        if lower.contains("ching") || lower.contains("coin") || lower.contains("cash") {
            Some(SfxType::Ching)
        } else if lower.contains("riser") || lower.contains("rise") || lower.contains("buildup") {
            Some(SfxType::Riser)
        } else if lower.contains("fall") || lower.contains("drop") {
            Some(SfxType::Faller)
        } else if lower.contains("whoosh") || lower.contains("swoosh") || lower.contains("swipe") {
            Some(SfxType::Whoosh)
        } else if lower.contains("pop") || lower.contains("blip") || lower.contains("bubble") {
            Some(SfxType::Pop)
        } else if lower.contains("boom") || lower.contains("bass") || lower.contains("impact") {
            Some(SfxType::Boom)
        } else if lower.contains("glitch") || lower.contains("error") || lower.contains("stutter") {
            Some(SfxType::Glitch)
        } else if lower.contains("click") || lower.contains("tap") || lower.contains("button") {
            Some(SfxType::Click)
        } else if lower.contains("sparkle") || lower.contains("magic") || lower.contains("shimmer") {
            Some(SfxType::Sparkle)
        } else if lower.contains("thud") || lower.contains("land") || lower.contains("heavy") {
            Some(SfxType::Thud)
        } else {
            None
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SfxType::Ching => "ching",
            SfxType::Riser => "riser",
            SfxType::Faller => "faller",
            SfxType::Whoosh => "whoosh",
            SfxType::Pop => "pop",
            SfxType::Boom => "boom",
            SfxType::Glitch => "glitch",
            SfxType::Click => "click",
            SfxType::Sparkle => "sparkle",
            SfxType::Thud => "thud",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            SfxType::Ching => "cash register / coin",
            SfxType::Riser => "tension build going up",
            SfxType::Faller => "whoosh going down",
            SfxType::Whoosh => "fast transition",
            SfxType::Pop => "bubble pop / blip",
            SfxType::Boom => "bass drop / impact",
            SfxType::Glitch => "digital stutter",
            SfxType::Click => "ui tap / button",
            SfxType::Sparkle => "magic shimmer",
            SfxType::Thud => "heavy impact",
        }
    }
}

// 4a. sfx library loaded from folder
pub struct SfxLibrary {
    sounds: HashMap<SfxType, Vec<PathBuf>>,
}

impl SfxLibrary {
    // 5a. scan a folder for sfx files
    pub fn load_from_folder(folder: impl AsRef<Path>) -> Self {
        let mut sounds: HashMap<SfxType, Vec<PathBuf>> = HashMap::new();
        
        // init empty vecs
        for sfx_type in SfxType::all() {
            sounds.insert(*sfx_type, Vec::new());
        }

        let folder = folder.as_ref();
        if !folder.exists() {
            log::warn!("sfx folder doesnt exist: {}", folder.display());
            return Self { sounds };
        }

        // walk the folder looking for audio files
        for entry in WalkDir::new(folder)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // check if its audio
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if !["mp3", "wav", "ogg", "m4a", "flac", "aac"].contains(&ext.as_str()) {
                    continue;
                }
            } else {
                continue;
            }

            // try to match to sfx type
            if let Some(filename) = path.file_stem() {
                let filename = filename.to_string_lossy();
                if let Some(sfx_type) = SfxType::from_filename(&filename) {
                    log::info!("found sfx: {} -> {:?}", path.display(), sfx_type);
                    sounds.get_mut(&sfx_type).unwrap().push(path.to_path_buf());
                }
            }
        }

        Self { sounds }
    }

    // 5b. get a random sound of a type
    pub fn get_random(&self, sfx_type: SfxType) -> Option<&PathBuf> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.sounds.get(&sfx_type)?.choose(&mut rng)
    }

    // 5c. which types have sounds available
    pub fn available_types(&self) -> Vec<SfxType> {
        self.sounds
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, _)| *k)
            .collect()
    }

    // 5d. check if a type has sounds
    pub fn has_type(&self, sfx_type: SfxType) -> bool {
        self.sounds
            .get(&sfx_type)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

// 6a. sfx placement event
#[derive(Debug, Clone)]
pub struct SfxEvent {
    pub sfx_type: SfxType,
    pub timestamp: f64,  // seconds into video
}

// ============================================
// TODO: auto sfx placement
// ============================================
// could add automatic sfx based on:
// - clip transitions (whoosh)
// - audio peaks in user video (boom)
// - random intervals (sparkle)
//
// for now sfx placement would be manual
// or we add at every clip transition
