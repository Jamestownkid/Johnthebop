<!--
  1a. SetupWizard.svelte - first run setup helper
  1b. shows when ffmpeg isnt installed
  1c. gives users easy instructions to fix it
  made this cuz users kept getting confused why nothing worked
-->

<script>
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import { dependencies, checkDependencies } from '../stores/app.js';
  
  const dispatch = createEventDispatcher();
  
  let checking = false;
  let copied = null;
  
  // 2a. copy command to clipboard
  async function copyCommand(cmd) {
    try {
      await navigator.clipboard.writeText(cmd);
      copied = cmd;
      setTimeout(() => copied = null, 2000);
    } catch (err) {
      console.error('failed to copy:', err);
    }
  }
  
  // 2b. recheck dependencies after user installs stuff
  async function recheckDeps() {
    checking = true;
    await checkDependencies();
    checking = false;
    
    // if all good now, close the wizard
    if ($dependencies.allGood) {
      dispatch('complete');
    }
  }
  
  // 2c. skip setup (use local files mode only)
  function skipSetup() {
    dispatch('skip');
  }
</script>

{#if !$dependencies.allGood && $dependencies.checked}
  <div class="setup-overlay">
    <div class="setup-card">
      <div class="setup-header">
        <h2>Quick Setup</h2>
        <p>GODZ needs a couple things installed to work. This only takes a minute.</p>
      </div>
      
      <div class="deps-list">
        <!-- FFmpeg -->
        <div class="dep-item" class:installed={$dependencies.ffmpegInstalled}>
          <div class="dep-header">
            <span class="dep-status">
              {#if $dependencies.ffmpegInstalled}
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
              {:else}
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="12" y1="8" x2="12" y2="12"/>
                  <line x1="12" y1="16" x2="12.01" y2="16"/>
                </svg>
              {/if}
            </span>
            <span class="dep-name">FFmpeg</span>
            <span class="dep-badge" class:required={true}>Required</span>
          </div>
          
          {#if !$dependencies.ffmpegInstalled}
            <p class="dep-desc">FFmpeg does all the video processing magic. Install it with one command:</p>
            
            <div class="install-commands">
              <div class="cmd-group">
                <span class="cmd-label">Ubuntu / Debian:</span>
                <div class="cmd-box">
                  <code>sudo apt install ffmpeg</code>
                  <button class="btn-copy" on:click={() => copyCommand('sudo apt install ffmpeg')}>
                    {copied === 'sudo apt install ffmpeg' ? 'Copied!' : 'Copy'}
                  </button>
                </div>
              </div>
              
              <div class="cmd-group">
                <span class="cmd-label">Fedora:</span>
                <div class="cmd-box">
                  <code>sudo dnf install ffmpeg</code>
                  <button class="btn-copy" on:click={() => copyCommand('sudo dnf install ffmpeg')}>
                    {copied === 'sudo dnf install ffmpeg' ? 'Copied!' : 'Copy'}
                  </button>
                </div>
              </div>
              
              <div class="cmd-group">
                <span class="cmd-label">Arch:</span>
                <div class="cmd-box">
                  <code>sudo pacman -S ffmpeg</code>
                  <button class="btn-copy" on:click={() => copyCommand('sudo pacman -S ffmpeg')}>
                    {copied === 'sudo pacman -S ffmpeg' ? 'Copied!' : 'Copy'}
                  </button>
                </div>
              </div>
            </div>
          {:else}
            <p class="dep-installed">FFmpeg is installed and ready to go!</p>
          {/if}
        </div>
        
        <!-- yt-dlp -->
        <div class="dep-item" class:installed={$dependencies.ytdlpInstalled}>
          <div class="dep-header">
            <span class="dep-status">
              {#if $dependencies.ytdlpInstalled}
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
              {:else}
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                </svg>
              {/if}
            </span>
            <span class="dep-name">yt-dlp</span>
            <span class="dep-badge optional">Optional</span>
          </div>
          
          {#if !$dependencies.ytdlpInstalled}
            <p class="dep-desc">Needed for downloading YouTube videos. Skip if youre using local files.</p>
            
            <div class="install-commands">
              <div class="cmd-group">
                <span class="cmd-label">Any system (with pip):</span>
                <div class="cmd-box">
                  <code>pip install yt-dlp</code>
                  <button class="btn-copy" on:click={() => copyCommand('pip install yt-dlp')}>
                    {copied === 'pip install yt-dlp' ? 'Copied!' : 'Copy'}
                  </button>
                </div>
              </div>
            </div>
          {:else}
            <p class="dep-installed">yt-dlp is installed - YouTube downloads will work!</p>
          {/if}
        </div>
      </div>
      
      <div class="setup-actions">
        <button class="btn-primary" on:click={recheckDeps} disabled={checking}>
          {#if checking}
            <span class="spinner"></span>
            Checking...
          {:else}
            Check Again
          {/if}
        </button>
        
        {#if $dependencies.ffmpegInstalled && !$dependencies.ytdlpInstalled}
          <button class="btn-secondary" on:click={skipSetup}>
            Continue Without YouTube Downloads
          </button>
        {/if}
      </div>
      
      {#if $dependencies.gpuEncoder && $dependencies.gpuEncoder !== 'CPU'}
        <div class="gpu-notice">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
          </svg>
          GPU acceleration detected: {$dependencies.gpuEncoder}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .setup-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: var(--space-lg);
  }
  
  .setup-card {
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    max-width: 600px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
  }
  
  .setup-header {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border-color);
    text-align: center;
  }
  
  .setup-header h2 {
    font-size: 24px;
    font-weight: 600;
    color: var(--accent-primary);
    margin-bottom: var(--space-sm);
  }
  
  .setup-header p {
    color: var(--text-secondary);
    font-size: 14px;
  }
  
  .deps-list {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }
  
  .dep-item {
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: var(--space-md);
    border: 1px solid var(--border-color);
  }
  
  .dep-item.installed {
    border-color: var(--success);
    background: rgba(74, 222, 128, 0.05);
  }
  
  .dep-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }
  
  .dep-status {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }
  
  .dep-item.installed .dep-status {
    color: var(--success);
  }
  
  .dep-item:not(.installed) .dep-status {
    color: var(--warning);
  }
  
  .dep-name {
    font-weight: 600;
    font-size: 16px;
  }
  
  .dep-badge {
    font-size: 10px;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .dep-badge.required {
    background: rgba(248, 113, 113, 0.2);
    color: var(--error);
  }
  
  .dep-badge.optional {
    background: rgba(96, 165, 250, 0.2);
    color: var(--info);
  }
  
  .dep-desc {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: var(--space-md);
  }
  
  .dep-installed {
    color: var(--success);
    font-size: 13px;
    font-weight: 500;
  }
  
  .install-commands {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  
  .cmd-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  
  .cmd-label {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
  }
  
  .cmd-box {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
    padding: var(--space-xs) var(--space-sm);
  }
  
  .cmd-box code {
    flex: 1;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    color: var(--accent-primary);
  }
  
  .btn-copy {
    font-size: 11px;
    padding: 4px 10px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }
  
  .btn-copy:hover {
    background: var(--accent-primary);
    color: var(--bg-primary);
  }
  
  .setup-actions {
    padding: var(--space-lg);
    border-top: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  
  .setup-actions .btn-primary,
  .setup-actions .btn-secondary {
    width: 100%;
    padding: var(--space-md);
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
  }
  
  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--bg-primary);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  .gpu-notice {
    margin: 0 var(--space-lg) var(--space-lg);
    padding: var(--space-sm) var(--space-md);
    background: rgba(0, 212, 170, 0.1);
    border-radius: var(--radius-sm);
    color: var(--accent-primary);
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }
  
  @keyframes spin { to { transform: rotate(360deg); } }
</style>

