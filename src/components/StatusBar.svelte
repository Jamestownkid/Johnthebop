<!--
  1a. StatusBar.svelte - bottom bar with status info
  1b. shows dependency status and active jobs
  1c. also displays errors when stuff breaks
-->

<script>
  import { dependencies, globalError, clearError } from '../stores/app.js';
  import { activeJobs } from '../stores/jobs.js';
</script>

<footer class="status-bar">
  <div class="status-left">
    <span class="status-item" title="FFmpeg - required for video processing">
      <span 
        class="status-dot"
        class:dot-success={$dependencies.ffmpegInstalled}
        class:dot-error={$dependencies.checked && !$dependencies.ffmpegInstalled}
      ></span>
      FFmpeg
    </span>
    <span class="status-item" title="yt-dlp - needed for YouTube downloads">
      <span 
        class="status-dot"
        class:dot-success={$dependencies.ytdlpInstalled}
        class:dot-warning={$dependencies.checked && !$dependencies.ytdlpInstalled}
      ></span>
      yt-dlp
    </span>
    
    {#if $dependencies.gpuEncoder && $dependencies.gpuEncoder !== 'unknown'}
      <span class="status-item encoder-item" title="Video encoder being used">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="4" y="4" width="16" height="16" rx="2"/>
          <rect x="9" y="9" width="6" height="6"/>
        </svg>
        {$dependencies.gpuEncoder.includes('GPU') ? 'GPU' : 'CPU'}
      </span>
    {/if}
  </div>
  
  <div class="status-center">
    {#if $activeJobs.length > 0}
      <span class="active-badge">
        <span class="pulse-dot"></span>
        {$activeJobs.length} {$activeJobs.length === 1 ? 'video' : 'videos'} processing
      </span>
    {:else}
      <span class="idle-text">Ready to create</span>
    {/if}
  </div>
  
  <div class="status-right">
    {#if $globalError}
      <div class="error-toast">
        <span class="error-text">{$globalError}</span>
        <button class="toast-dismiss" on:click={clearError}>x</button>
      </div>
    {/if}
  </div>
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) var(--space-md);
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    height: 36px;
    flex-shrink: 0;
    font-size: 12px;
  }
  
  .status-left {
    display: flex;
    gap: var(--space-md);
  }
  
  .status-item {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    color: var(--text-secondary);
    cursor: default;
  }
  
  .encoder-item {
    margin-left: var(--space-sm);
    padding-left: var(--space-sm);
    border-left: 1px solid var(--border-color);
  }
  
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted);
  }
  
  .dot-success {
    background: var(--success);
    box-shadow: 0 0 6px var(--success);
  }
  
  .dot-error {
    background: var(--error);
    box-shadow: 0 0 6px var(--error);
  }
  
  .dot-warning {
    background: var(--warning);
  }
  
  .status-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }
  
  .active-badge {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    background: rgba(0, 255, 213, 0.15);
    color: var(--accent-primary);
    padding: 4px 12px;
    border-radius: var(--radius-full);
    font-weight: 500;
    font-size: 11px;
  }
  
  .pulse-dot {
    width: 6px;
    height: 6px;
    background: var(--accent-primary);
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }
  
  .idle-text {
    color: var(--text-muted);
    font-size: 11px;
  }
  
  .status-right {
    min-width: 200px;
    display: flex;
    justify-content: flex-end;
  }
  
  .error-toast {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    background: rgba(239, 68, 68, 0.15);
    color: var(--error);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    max-width: 300px;
    animation: slideUp 0.2s ease;
  }
  
  .error-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 11px;
  }
  
  .toast-dismiss {
    color: var(--error);
    font-size: 14px;
    line-height: 1;
    padding: 0 4px;
    opacity: 0.7;
  }
  
  .toast-dismiss:hover {
    opacity: 1;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.8); }
  }
  
  @keyframes slideUp {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
