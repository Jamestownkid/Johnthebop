// 1a. app.js - app-level state and helpers
// 1b. dependency status, directories, global errors
// 1c. the glue that holds everything together kinda

import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

// 2a. dependency status - ffmpeg, yt-dlp, gpu encoder
export const dependencies = writable({
  ffmpegInstalled: false,
  ytdlpInstalled: false,
  allGood: false,
  checked: false,
  gpuEncoder: 'unknown',
});

// 2b. app directories
export const appDirs = writable({
  temp: null,
  exports: null,
});

// 2c. loading state for when stuff is happening
export const isLoading = writable(false);

// 2d. global error message - shows in status bar
export const globalError = writable(null);

// 3a. check what dependencies are installed
// called on app startup
export async function checkDependencies() {
  try {
    const status = await invoke('check_dependencies');
    dependencies.set({
      ffmpegInstalled: status.ffmpeg_installed,
      ytdlpInstalled: status.ytdlp_installed,
      allGood: status.all_good,
      checked: true,
      gpuEncoder: status.gpu_encoder || 'CPU',
    });
    return status;
  } catch (err) {
    console.error('failed to check dependencies:', err);
    dependencies.update(d => ({ ...d, checked: true }));
    return null;
  }
}

// 3b. get app directories for temp and export files
export async function getAppDirs() {
  try {
    const dirs = await invoke('get_app_dirs');
    appDirs.set(dirs);
    return dirs;
  } catch (err) {
    console.error('failed to get app dirs:', err);
    return null;
  }
}

// 3c. validate a youtube url
export async function validateYoutubeUrl(url) {
  try {
    return await invoke('validate_youtube_url', { url });
  } catch (err) {
    console.error('failed to validate url:', err);
    return false;
  }
}

// 4a. show a global error toast
// auto-clears after duration (default 5s)
export function showError(message, duration = 5000) {
  globalError.set(message);
  
  if (duration > 0) {
    setTimeout(() => {
      globalError.set(null);
    }, duration);
  }
}

// 4b. manually clear the error
export function clearError() {
  globalError.set(null);
}
