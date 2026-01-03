# GODZ - B-Roll Video Creator

Create professional-looking B-Roll composite videos in minutes. Just paste YouTube links or select local files, pick your settings, and let GODZ do the rest.

Built for content creators who want to make engaging videos without dealing with copyright strikes or complex editing software.

## What It Does

1. **Get B-Roll** - Paste YouTube links or select local video files
2. **Add Your Video** - Select your talking head / main content
3. **Pick Layout** - Choose how B-Roll appears (top, bottom, corner, side-by-side)
4. **Export** - Get a ready-to-upload video for YouTube, TikTok, or Instagram

The B-Roll gets automatically chopped into short clips and mixed together, making it essentially "original" content that wont trigger copyright claims.

## Features

- **Multiple Input Modes** - YouTube downloads or local files
- **Overlay Positions** - Top/bottom split, picture-in-picture, side-by-side
- **Platform Presets** - YouTube (16:9), TikTok (9:16), Instagram (4:5)
- **Custom Dimensions** - Set your own output size
- **GPU Acceleration** - Uses NVIDIA/AMD/Intel hardware encoding when available
- **Job Queue** - Process multiple videos, track progress in real-time
- **No Command Line** - Everything is point and click

## Screenshot

```
+--------------------------------------------------+
|  GODZ - B-Roll Creator                      v1.0 |
+------------------------+-------------------------+
|  Create New Video      |  Your Jobs              |
|                        |                         |
|  [YouTube] [Local]     |  #abc123 - YouTube      |
|                        |  Processing... 45%      |
|  Paste links here...   |  [================    ] |
|                        |                         |
|  Your Video:           |  #def456 - Complete     |
|  [Browse...]           |  [Open Video]           |
|                        |                         |
|  B-Roll Position:      |                         |
|  [Top v]               |                         |
|                        |                         |
|  [=== 50% ===]         |                         |
|                        |                         |
|  [Create Video]        |                         |
+------------------------+-------------------------+
|  FFmpeg OK  |  yt-dlp OK  |  Ready to create     |
+--------------------------------------------------+
```

## Installation

### Download Release (Easiest)

Go to [Releases](https://github.com/Jamestownkid/Johnthebop/releases) and download:
- `.AppImage` for Linux
- `.deb` for Ubuntu/Debian
- `.dmg` for macOS
- `.exe` / `.msi` for Windows

### Build From Source

```bash
# clone the repo
git clone https://github.com/Jamestownkid/Johnthebop.git
cd Johnthebop

# install dependencies
npm install

# run in dev mode
npm run tauri dev

# or build for production
npm run tauri build
```

## Requirements

### FFmpeg (Required)

GODZ uses FFmpeg for all video processing. Install it first:

**Ubuntu/Debian:**
```bash
sudo apt install ffmpeg
```

**Fedora:**
```bash
sudo dnf install ffmpeg
```

**Arch:**
```bash
sudo pacman -S ffmpeg
```

**macOS:**
```bash
brew install ffmpeg
```

**Windows:**
Download from [ffmpeg.org](https://ffmpeg.org/download.html) and add to PATH.

### yt-dlp (Optional)

Only needed if you want to download from YouTube:

```bash
pip install yt-dlp
```

Or use Local Files mode if you already have your B-Roll downloaded.

## How It Avoids Copyright Claims

Based on research into how Content ID works:

| Method | Why It Works |
|--------|--------------|
| 4 sec max clips | Most owners set detection thresholds at 7-10 seconds |
| No B-Roll audio | Audio fingerprinting is more sensitive than video |
| Source rotation | Prevents cumulative matching from single source |
| Random ordering | Creates unique visual fingerprint each time |

This isnt foolproof, but it works for most cases. The key is creating something genuinely new from the combination of sources.

## Tech Stack

- **Rust** - Backend, handles video processing
- **Tauri** - Desktop app framework (way lighter than Electron)
- **Svelte** - Frontend UI
- **FFmpeg** - Video encoding and compositing

## Folder Structure

```
godz/
+-- src/                   # Frontend (Svelte)
|   +-- components/        # UI components
|   +-- stores/            # State management
|   +-- styles/            # CSS
+-- src-tauri/             # Backend (Rust)
|   +-- src/
|       +-- main.rs        # Entry point
|       +-- processor.rs   # FFmpeg operations
|       +-- downloader.rs  # yt-dlp wrapper
|       +-- scrambler.rs   # Clip mixing logic
|       +-- jobs.rs        # Job queue
+-- .github/workflows/     # CI/CD
```

## Troubleshooting

**FFmpeg not found**
- Make sure its installed and in your PATH
- Try running `ffmpeg -version` in terminal

**YouTube download fails**
- Switch to Local Files mode
- Update yt-dlp: `pip install -U yt-dlp`
- Some videos may be geo-restricted

**Video looks weird**
- Check that your input files arent corrupted
- Try a different output format
- Make sure aspect ratios are similar

**Processing is slow**
- Check if GPU acceleration is detected (shown in status bar)
- Close other programs using GPU
- Use smaller input files for testing

## Contributing

PRs welcome! Please keep the code style consistent:
- Comments should be casual but helpful
- Use the 1a, 1b, 1c numbering format
- Test your changes before submitting

## License

MIT - do whatever you want with it

---

**GitHub:** https://github.com/Jamestownkid/Johnthebop

Made with late nights and too much caffeine
