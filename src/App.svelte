<!--
  1a. App.svelte - main app component for godz
  1b. two panel layout: input on left, jobs on right
  1c. also shows setup wizard if dependencies missing
-->

<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import Header from './components/Header.svelte';
  import InputPanel from './components/InputPanel.svelte';
  import JobsPanel from './components/JobsPanel.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import SetupWizard from './components/SetupWizard.svelte';
  
  import { jobs, updateJobs } from './stores/jobs.js';
  import { dependencies, checkDependencies } from './stores/app.js';

  // 2a. track if setup is done
  let setupComplete = false;
  
  // 2b. poll interval for job updates
  let pollInterval;
  
  onMount(async () => {
    // check dependencies first
    await checkDependencies();
    
    // if ffmpeg is there, setup is basically done
    if ($dependencies.ffmpegInstalled) {
      setupComplete = true;
    }
    
    // start polling for job updates
    pollInterval = setInterval(async () => {
      await updateJobs();
    }, 1000);
    
    return () => {
      if (pollInterval) clearInterval(pollInterval);
    };
  });
  
  // 3a. handle setup wizard completion
  function handleSetupComplete() {
    setupComplete = true;
  }
  
  function handleSetupSkip() {
    // user wants to skip (will use local files only)
    setupComplete = true;
  }
</script>

<div class="app">
  <Header />
  
  <main class="main-content">
    <div class="panel input-panel">
      <InputPanel />
    </div>
    
    <div class="panel jobs-panel">
      <JobsPanel />
    </div>
  </main>
  
  <StatusBar />
  
  <!-- Setup wizard shows if deps not installed -->
  {#if !setupComplete && $dependencies.checked}
    <SetupWizard 
      on:complete={handleSetupComplete}
      on:skip={handleSetupSkip}
    />
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
  }
  
  .main-content {
    flex: 1;
    display: flex;
    gap: var(--space-md);
    padding: var(--space-md);
    overflow: hidden;
  }
  
  .panel {
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    transition: box-shadow var(--transition-normal);
  }
  
  .panel:hover {
    box-shadow: var(--shadow-md);
  }
  
  .input-panel {
    flex: 0 0 55%;
    min-width: 400px;
  }
  
  .jobs-panel {
    flex: 1;
    min-width: 320px;
  }
  
  /* responsive - stack on smaller screens */
  @media (max-width: 900px) {
    .main-content {
      flex-direction: column;
    }
    
    .input-panel {
      flex: 0 0 auto;
      min-width: unset;
    }
    
    .jobs-panel {
      flex: 1;
      min-width: unset;
      min-height: 200px;
    }
  }
</style>
