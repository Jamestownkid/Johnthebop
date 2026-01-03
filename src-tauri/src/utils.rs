// 1a. utils.rs - random helper functions
// 1b. stuff that doesnt fit anywhere else
// 1c. validators, formatters, file helpers

use std::path::Path;

// 2a. check if url looks like a valid youtube link
// not perfect but catches obvious mistakes
pub fn is_valid_youtube_url(url: &str) -> bool {
    let patterns = [
        "youtube.com/watch",
        "youtu.be/",
        "youtube.com/shorts/",
        "youtube.com/v/",
        "youtube.com/embed/",
    ];
    
    patterns.iter().any(|p| url.contains(p))
}

// 2b. check if file looks like a video we can process
pub fn is_valid_video_file(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    
    let valid_exts = ["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv"];
    
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| valid_exts.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

// 3a. format seconds as mm:ss
pub fn format_duration(seconds: f64) -> String {
    let mins = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    format!("{}:{:02}", mins, secs)
}

// 3b. format bytes as human readable size
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// 4a. sanitize filename - remove sketchy chars
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_' || *c == '.')
        .collect::<String>()
        .trim()
        .to_string()
}

// 4b. generate unique temp filename
pub fn temp_filename(prefix: &str, extension: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    format!("{}_{}.{}", prefix, id, extension)
}

// 5a. cleanup old temp files
// deletes files older than max_age_hours
pub fn cleanup_old_temp_files(temp_dir: &Path, max_age_hours: u64) -> std::io::Result<usize> {
    use std::time::{SystemTime, Duration};
    
    let max_age = Duration::from_secs(max_age_hours * 3600);
    let now = SystemTime::now();
    let mut deleted = 0;
    
    if !temp_dir.exists() {
        return Ok(0);
    }
    
    for entry in std::fs::read_dir(temp_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > max_age {
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(&path);
                        } else {
                            let _ = std::fs::remove_file(&path);
                        }
                        deleted += 1;
                    }
                }
            }
        }
    }
    
    Ok(deleted)
}

// 6a. tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_url_validation() {
        assert!(is_valid_youtube_url("https://www.youtube.com/watch?v=abc123"));
        assert!(is_valid_youtube_url("https://youtu.be/abc123"));
        assert!(is_valid_youtube_url("https://youtube.com/shorts/abc123"));
        assert!(!is_valid_youtube_url("https://vimeo.com/123"));
        assert!(!is_valid_youtube_url("not a url"));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(65.0), "1:05");
        assert_eq!(format_duration(3661.0), "61:01");
        assert_eq!(format_duration(30.0), "0:30");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello world");
        assert_eq!(sanitize_filename("bad/name\\here"), "badnamehere");
        assert_eq!(sanitize_filename("  spaces  "), "spaces");
    }
}
