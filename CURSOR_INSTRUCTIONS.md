# GODZ - Complete Build Instructions for Cursor

yo cursor, this is the full breakdown of what you need to do to turn this into a finished, publish-ready app. read this whole thing before you start coding.

---

## PART 1: PROJECT OVERVIEW

### what this app does
- user records themselves talking (the "talking head" video)
- user provides b-roll footage (either youtube links OR local files)
- app chops up the b-roll into short clips (under 4 seconds)
- app composites: b-roll on TOP, user video on BOTTOM
- outputs a ready-to-upload video for youtube/tiktok/instagram

### the name
call the app **GODZ** (not broll-scrambler anymore)

### github repo
push everything to: https://github.com/Jamestownkid/Johnthebop

---

## PART 2: CRITICAL FIXES NEEDED

### 2a. FALLBACK SYSTEM FOR B-ROLL INPUT

the yt-dlp download might not work for everyone (network issues, blocked, whatever). we need a fallback:

```
PRIMARY MODE: paste youtube links -> download with yt-dlp
FALLBACK MODE: if download fails OR user prefers -> select local video files
```

in the UI, add a toggle or tab system:
```svelte
<!-- in InputPanel.svelte, add this -->
<div class="input-mode-tabs">
  <button 
    class:active={inputMode === 'youtube'} 
    on:click={() => inputMode = 'youtube'}
  >
    YouTube Links
  </button>
  <button 
    class:active={inputMode === 'local'} 
    on:click={() => inputMode = 'local'}
  >
    Local Files
  </button>
</div>

{#if inputMode === 'youtube'}
  <!-- existing youtube links textarea -->
{:else}
  <!-- file picker for multiple local videos -->
  <div class="local-files-picker">
    <button on:click={selectLocalBroll}>Select B-Roll Videos</button>
    {#each localBrollFiles as file}
      <div class="file-item">{file}</div>
    {/each}
  </div>
{/if}
```

in the rust backend, modify the job system:
```rust
// in jobs.rs, update JobConfig
pub struct JobConfig {
    // EITHER youtube links OR local paths, not both
    pub broll_source: BrollSource,
    pub user_video_path: String,
    pub output_format: OutputFormat,
    // ... rest of fields
}

pub enum BrollSource {
    YouTube(Vec<String>),      // list of youtube URLs
    LocalFiles(Vec<String>),   // list of local file paths
}
```

### 2b. YT-DLP BUNDLING

users shouldnt have to install yt-dlp separately. bundle it with the app:

option 1 - download on first run:
```rust
// in main.rs or a new setup.rs
async fn ensure_ytdlp_available() -> Result<PathBuf, String> {
    let app_dir = get_app_data_dir()?;
    let ytdlp_path = app_dir.join("bin").join("yt-dlp");
    
    if ytdlp_path.exists() {
        return Ok(ytdlp_path);
    }
    
    // download yt-dlp binary
    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";
    download_file(url, &ytdlp_path).await?;
    
    // make executable
    #[cfg(unix)]
    std::fs::set_permissions(&ytdlp_path, std::fs::Permissions::from_mode(0o755))?;
    
    Ok(ytdlp_path)
}
```

option 2 - include in app bundle (tauri.conf.json):
```json
{
  "tauri": {
    "bundle": {
      "resources": ["bin/yt-dlp"]
    }
  }
}
```

### 2c. FFMPEG HANDLING

same deal - users might not have ffmpeg. options:

1. check on startup, show friendly install instructions
2. bundle a static ffmpeg binary
3. use ffmpeg-next rust crate instead of shelling out

for now, do option 1 but make the error message SUPER clear:

```svelte
<!-- in App.svelte or a new SetupWizard.svelte -->
{#if !$dependencies.ffmpegInstalled}
  <div class="setup-overlay">
    <div class="setup-card">
      <h2>One-Time Setup Required</h2>
      <p>GODZ needs FFmpeg to process videos. It's free and takes 30 seconds:</p>
      
      <div class="install-instructions">
        <h3>Ubuntu/Debian:</h3>
        <code>sudo apt install ffmpeg</code>
        <button on:click={() => copyToClipboard('sudo apt install ffmpeg')}>Copy</button>
        
        <h3>Fedora:</h3>
        <code>sudo dnf install ffmpeg</code>
        
        <h3>Arch:</h3>
        <code>sudo pacman -S ffmpeg</code>
      </div>
      
      <button class="btn-primary" on:click={recheckDependencies}>
        I've Installed It - Check Again
      </button>
    </div>
  </div>
{/if}
```

---

## PART 3: USER EXPERIENCE IMPROVEMENTS

### 3a. FIRST-RUN WIZARD

when app opens for first time, show a quick wizard:
1. check dependencies (ffmpeg, yt-dlp)
2. ask where to save exports (default: ~/Videos/GODZ/)
3. optional: import existing b-roll library

### 3b. PROJECT SYSTEM

users shouldnt have to re-enter everything each time. add projects:

```
~/GODZ/
├── projects/
│   ├── my-first-video/
│   │   ├── project.json       # settings, links, paths
│   │   ├── broll/             # downloaded or copied b-roll
│   │   ├── clips/             # processed clips
│   │   └── exports/           # final outputs
│   └── another-project/
└── settings.json              # global settings
```

### 3c. DRAG AND DROP EVERYWHERE

- drag youtube links into the app
- drag video files into the app
- drag to reorder clips (future feature)

```svelte
<div 
  class="drop-zone"
  on:dragover|preventDefault={() => isDragging = true}
  on:dragleave={() => isDragging = false}
  on:drop|preventDefault={handleDrop}
  class:dragging={isDragging}
>
  <p>Drop videos here or paste YouTube links</p>
</div>
```

### 3d. PROGRESS VISUALIZATION

show actual progress, not just "processing...":

```
Downloading B-Roll... (2/5)
  ├── video1.mp4 ✓
  ├── video2.mp4 ✓
  ├── video3.mp4 ◌ downloading 45%
  ├── video4.mp4 ○ queued
  └── video5.mp4 ○ queued

Cutting Clips... (15/32)
[████████████░░░░░░░░] 47%

Compositing Final Video...
[████░░░░░░░░░░░░░░░░] 12%
ETA: 2:34
```

---

## PART 4: COMMENT STYLE GUIDE

### dont sound like AI wrote it

BAD (sounds like AI):
```rust
/// This function processes the video file and returns the result.
/// It takes a path parameter and performs necessary operations.
fn process_video(path: &Path) -> Result<Video, Error> {
```

GOOD (sounds like a person):
```rust
// 1a. process_video - does the actual ffmpeg stuff
// 1b. takes a path, spits out processed video
// 1c. might fail if file is corrupted or whatever
fn process_video(path: &Path) -> Result<Video, Error> {
```

### rules for comments:
- use lowercase mostly
- contractions are fine (dont, cant, shouldnt)
- slang is fine (gonna, wanna, kinda, lowkey, ngl)
- curse occasionally if frustrated (this is janky af, wtf is this api)
- admit when something is hacky
- reference why you did something, not just what
- use the 1a, 1b, 1c format for organization
- short comments > long comments

### examples of good comments:
```rust
// why tf does ffmpeg need this flag? idk but it breaks without it
args.push("-nostdin");

// 2a. this is probably overengineered but whatever
// 2b. basically we need to track which clips came from where
// 2c. so if content id flags something we know which source to remove
struct ClipMetadata {
    source_idx: usize,
    // ...
}

// lmao this took forever to figure out
// the youtube url can be in like 5 different formats
fn extract_video_id(url: &str) -> Option<String> {

// TODO: this is kinda slow, maybe cache it?
// but also who cares, it runs once per job
```

---

## PART 5: ICON AND BRANDING

### app icon requirements

create these files in `src-tauri/icons/`:
- 32x32.png
- 128x128.png
- 128x128@2x.png (256x256)
- icon.ico (windows)
- icon.icns (mac)

### design spec:
- dark background (#0a0a0a or #121212)
- main color: electric cyan/teal (#00ffd5 or #00d4aa)
- secondary: white or light gray
- style: minimal, techy, maybe slightly brutalist
- concept: could be stylized "G", film strips, split screen icon, or abstract

### generate icons programmatically:
```bash
# if you have imagemagick
convert -size 512x512 xc:'#121212' \
  -fill '#00ffd5' -draw "roundrectangle 80,50,430,220,20,20" \
  -fill '#ffffff' -draw "roundrectangle 80,290,430,460,20,20" \
  icon_base.png

# then resize
convert icon_base.png -resize 32x32 32x32.png
convert icon_base.png -resize 128x128 128x128.png
convert icon_base.png -resize 256x256 128x128@2x.png
```

or use figma, photoshop, whatever. just make it look good.

---

## PART 6: FILE STRUCTURE (FINAL)

```
godz/
├── .github/
│   └── workflows/
│       └── release.yml        # github actions for auto-build
├── src-tauri/
│   ├── src/
│   │   ├── main.rs            # entry, tauri commands
│   │   ├── downloader.rs      # yt-dlp wrapper + fallback
│   │   ├── processor.rs       # ffmpeg operations
│   │   ├── scrambler.rs       # clip mixing logic
│   │   ├── jobs.rs            # job queue
│   │   ├── sfx.rs             # sound effects
│   │   ├── projects.rs        # NEW: project management
│   │   ├── settings.rs        # NEW: user settings
│   │   └── utils.rs           # helpers
│   ├── icons/                 # app icons
│   ├── bin/                   # bundled binaries (yt-dlp)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── components/
│   │   ├── Header.svelte
│   │   ├── InputPanel.svelte  # updated with local file support
│   │   ├── JobsPanel.svelte
│   │   ├── StatusBar.svelte
│   │   ├── SetupWizard.svelte # NEW: first-run setup
│   │   ├── ProjectPicker.svelte # NEW
│   │   └── ProgressDisplay.svelte # NEW: detailed progress
│   ├── stores/
│   │   ├── jobs.js
│   │   ├── app.js
│   │   ├── projects.js        # NEW
│   │   └── settings.js        # NEW
│   ├── lib/
│   │   └── tauri.js
│   ├── styles/
│   │   └── global.css
│   ├── App.svelte
│   └── main.js
├── public/
│   └── favicon.svg
├── scripts/
│   └── build-icons.sh         # generates all icon sizes
├── package.json
├── vite.config.js
├── svelte.config.js
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

---

## PART 7: GITHUB SETUP

### repo structure for https://github.com/Jamestownkid/Johnthebop

1. create good README.md with:
   - screenshot/gif of app
   - features list
   - installation instructions
   - usage guide
   - build from source instructions

2. add releases with pre-built binaries:
   - .AppImage for Linux
   - .deb for Debian/Ubuntu
   - .rpm for Fedora
   - .dmg for macOS (if possible)
   - .exe/.msi for Windows (if possible)

3. github actions workflow for auto-building:

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  build-linux:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      
      - name: Install deps
        run: |
          sudo apt update
          sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install frontend deps
        run: npm install
      
      - name: Build
        run: npm run tauri build
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: linux-build
          path: |
            src-tauri/target/release/bundle/appimage/*.AppImage
            src-tauri/target/release/bundle/deb/*.deb
```

---

## PART 8: THINGS THAT MIGHT BE BROKEN

### known issues to check and fix:

1. **shell import in JobsPanel.svelte** - might need:
```javascript
import { shell } from '@tauri-apps/api';
// change to:
import { open } from '@tauri-apps/api/shell';
```

2. **futures crate** - already added to Cargo.toml but verify it compiles

3. **tauri dialog API** - make sure the version matches:
```javascript
// old api
import { open } from '@tauri-apps/api/dialog';
// might need
import { open } from '@tauri-apps/plugin-dialog';
```

4. **path handling on windows** - use path.join() not string concat

5. **yt-dlp output parsing** - test with actual videos

6. **ffmpeg filter syntax** - test the composite filter actually works

---

## PART 9: TESTING CHECKLIST

before publishing, test these scenarios:

### happy path:
- [ ] paste 3 youtube links
- [ ] select a 2-minute user video
- [ ] select youtube format
- [ ] click start
- [ ] job completes
- [ ] output video plays correctly
- [ ] b-roll is on top, user on bottom
- [ ] audio is from user video only

### fallback path:
- [ ] disconnect internet
- [ ] try youtube links (should fail gracefully)
- [ ] switch to local files mode
- [ ] select local video files
- [ ] job completes

### error handling:
- [ ] invalid youtube URL shows error
- [ ] corrupted video file shows error
- [ ] disk full shows error
- [ ] cancel job works
- [ ] can start new job after cancel

### edge cases:
- [ ] very short user video (10 seconds)
- [ ] very long user video (30 minutes)
- [ ] only 1 b-roll source
- [ ] 10+ b-roll sources
- [ ] different aspect ratios
- [ ] different framerates

---

## PART 10: FINAL ZIP

when everything is done, create the final zip:

```bash
# clean build artifacts first
rm -rf node_modules target dist

# create zip named godz-v1.0.0.zip
zip -r godz-v1.0.0.zip godz/ \
  -x "*.git*" \
  -x "*node_modules*" \
  -x "*target*" \
  -x "*.DS_Store"
```

the zip should be under 1MB (source only, no dependencies)

---

## PART 11: QUICK REFERENCE

### commands:
```bash
# dev mode
npm run tauri dev

# production build
npm run tauri build

# just frontend
npm run dev

# just compile rust
cd src-tauri && cargo build
```

### useful links:
- tauri docs: https://tauri.app/v1/guides/
- svelte docs: https://svelte.dev/docs
- ffmpeg filters: https://ffmpeg.org/ffmpeg-filters.html
- yt-dlp options: https://github.com/yt-dlp/yt-dlp#usage-and-options

---

## TL;DR

1. rename app to GODZ
2. add local file fallback for b-roll
3. bundle yt-dlp or auto-download it
4. add first-run setup wizard
5. make comments sound human (use slang, 1a/1b/1c format)
6. create proper icons
7. test everything
8. push to github
9. set up releases with github actions
10. create final zip

good luck homie, make it clean
