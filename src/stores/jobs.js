// 1a. jobs.js - state management for video jobs
// 1b. svelte stores are pretty dope tbh
// 1c. writable = can change, derived = computed from other stores

import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

// 2a. main jobs store - array of job objects
export const jobs = writable([]);

// 2b. currently selected job for details view
export const selectedJobId = writable(null);

// 3a. derived store for active (running) jobs
export const activeJobs = derived(jobs, ($jobs) => {
  return $jobs.filter(job => 
    !['Complete', 'Failed', 'Cancelled'].includes(job.state)
  );
});

// 3b. derived store for completed jobs
export const completedJobs = derived(jobs, ($jobs) => {
  return $jobs.filter(job => job.state === 'Complete');
});

// 4a. fetch all jobs from backend
// called every second by the polling in App.svelte
export async function updateJobs() {
  try {
    const allJobs = await invoke('get_all_jobs');
    jobs.set(allJobs);
  } catch (err) {
    console.error('failed to fetch jobs:', err);
  }
}

// 4b. start a new job with all the settings
// this is the main function users trigger
export async function startJob(config) {
  try {
    const jobId = await invoke('start_job', {
      youtubeLinks: config.youtubeLinks || [],
      localBrollPaths: config.localBrollPaths || null,
      userVideoPath: config.userVideoPath,
      outputFormat: config.outputFormat,
      overlayPosition: config.overlayPosition || 'top',
      customWidth: config.customWidth || null,
      customHeight: config.customHeight || null,
      splitRatio: config.splitRatio || null,
      pipScale: config.pipScale || null,
      sfxFolder: config.sfxFolder || null,
    });
    
    // refresh jobs list right away
    await updateJobs();
    
    return { success: true, jobId };
  } catch (err) {
    console.error('failed to start job:', err);
    return { success: false, error: err };
  }
}

// 4c. cancel a running job
export async function cancelJob(jobId) {
  try {
    await invoke('cancel_job', { jobId });
    await updateJobs();
    return { success: true };
  } catch (err) {
    console.error('failed to cancel job:', err);
    return { success: false, error: err };
  }
}

// 4d. get status of a specific job
export async function getJobStatus(jobId) {
  try {
    return await invoke('get_job_status', { jobId });
  } catch (err) {
    console.error('failed to get job status:', err);
    return null;
  }
}
