// 1a. tauri.js - helpers for tauri API
// 1b. wraps common tauri operations
// 1c. makes it easier to call from components

import { invoke } from '@tauri-apps/api/tauri';
import { open, save } from '@tauri-apps/api/dialog';
import { homeDir, join } from '@tauri-apps/api/path';

// 2a. pick a video file
export async function pickVideoFile() {
  const selected = await open({
    multiple: false,
    filters: [{
      name: 'Video Files',
      extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv']
    }]
  });
  return selected;
}

// 2b. pick a folder
export async function pickFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  return selected;
}

// 2c. pick multiple video files
export async function pickVideoFiles() {
  const selected = await open({
    multiple: true,
    filters: [{
      name: 'Video Files',
      extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv']
    }]
  });
  return selected || [];
}

// 3a. get default output location
export async function getDefaultOutputDir() {
  try {
    const home = await homeDir();
    return await join(home, 'Videos', 'BRollScrambler');
  } catch (err) {
    console.error('failed to get home dir:', err);
    return null;
  }
}

// 3b. wrapper for invoke with error handling
export async function safeInvoke(command, args = {}) {
  try {
    return { success: true, data: await invoke(command, args) };
  } catch (err) {
    console.error(`invoke ${command} failed:`, err);
    return { success: false, error: err };
  }
}
