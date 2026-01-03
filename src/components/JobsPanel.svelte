<!--
  1a. JobsPanel.svelte - shows all jobs and their progress
  1b. users can see whats running, whats done, whats broken
  1c. also lets em cancel or open completed videos
-->

<script>
  import { jobs, cancelJob, selectedJobId } from '../stores/jobs.js';
  import { open } from '@tauri-apps/api/shell';
  
  // 2a. format timestamp nicely
  function formatTime(isoString) {
    if (!isoString) return '-';
    const date = new Date(isoString);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  
  // 2b. get color class based on state
  function getStatusClass(state) {
    switch (state) {
      case 'Complete': return 'status-success';
      case 'Failed': return 'status-error';
      case 'Cancelled': return 'status-muted';
      default: return 'status-active';
    }
  }
  
  // 2c. get icon based on state
  function getStatusIcon(state) {
    switch (state) {
      case 'Complete': return '~';
      case 'Failed': return 'x';
      case 'Cancelled': return '-';
      case 'Queued': return 'o';
      default: return '>';
    }
  }
  
  // 3a. handle cancel button
  async function handleCancel(jobId, event) {
    event.stopPropagation();
    await cancelJob(jobId);
  }
  
  // 3b. open the output file in file manager
  async function openOutput(outputPath, event) {
    event.stopPropagation();
    if (outputPath) {
      try {
        await open(outputPath);
      } catch (err) {
        console.error('failed to open output:', err);
      }
    }
  }
  
  // 3c. get a friendly stage name
  function getShortStage(stage) {
    if (stage.length > 40) {
      return stage.substring(0, 37) + '...';
    }
    return stage;
  }
</script>

<div class="jobs-panel-content">
  <div class="panel-header">
    <h2>Your Jobs</h2>
    <span class="job-count">{$jobs.length}</span>
  </div>
  
  <div class="jobs-list">
    {#if $jobs.length === 0}
      <!-- empty state -->
      <div class="empty-state">
        <div class="empty-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <line x1="9" y1="3" x2="9" y2="21"/>
            <line x1="15" y1="3" x2="15" y2="21"/>
            <line x1="3" y1="9" x2="21" y2="9"/>
            <line x1="3" y1="15" x2="21" y2="15"/>
          </svg>
        </div>
        <p>No videos yet</p>
        <span class="empty-hint">Create your first video on the left</span>
      </div>
    {:else}
      <!-- job list -->
      {#each $jobs as job (job.id)}
        <div 
          class="job-item"
          class:selected={$selectedJobId === job.id}
          on:click={() => selectedJobId.set(job.id)}
          on:keypress={(e) => e.key === 'Enter' && selectedJobId.set(job.id)}
          role="button"
          tabindex="0"
        >
          <div class="job-header">
            <span class="job-status {getStatusClass(job.state)}">
              {getStatusIcon(job.state)}
            </span>
            <span class="job-id">#{job.id}</span>
            <span class="job-format">{job.output_format}</span>
            <span class="job-time">{formatTime(job.created_at)}</span>
          </div>
          
          <div class="job-info">
            <span class="job-overlay">{job.overlay_position}</span>
          </div>
          
          <div class="job-progress">
            <span class="progress-stage">{getShortStage(job.progress.stage)}</span>
            {#if job.state !== 'Complete' && job.state !== 'Failed' && job.state !== 'Cancelled'}
              <div class="progress-bar">
                <div 
                  class="progress-fill" 
                  style="width: {job.progress.percent}%"
                ></div>
              </div>
              <span class="progress-percent">{Math.round(job.progress.percent)}%</span>
            {/if}
          </div>
          
          <div class="job-actions">
            {#if job.state === 'Complete' && job.output_path}
              <button 
                class="btn-small btn-success"
                on:click={(e) => openOutput(job.output_path, e)}
              >
                Open Video
              </button>
            {:else if !['Complete', 'Failed', 'Cancelled'].includes(job.state)}
              <button 
                class="btn-small btn-danger"
                on:click={(e) => handleCancel(job.id, e)}
              >
                Cancel
              </button>
            {:else if job.state === 'Failed'}
              <span class="failed-label">Failed</span>
            {/if}
          </div>
          
          {#if job.error}
            <div class="job-error">
              {job.error}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .jobs-panel-content {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  
  .panel-header {
    padding: var(--space-md);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }
  
  .panel-header h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--accent-primary);
  }
  
  .job-count {
    background: var(--bg-tertiary);
    padding: 2px 10px;
    border-radius: var(--radius-full);
    font-size: 12px;
    color: var(--text-secondary);
    font-weight: 500;
  }
  
  .jobs-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm);
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--text-muted);
    padding: var(--space-xl);
  }
  
  .empty-icon {
    margin-bottom: var(--space-md);
    opacity: 0.3;
    color: var(--accent-primary);
  }
  
  .empty-state p {
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--space-xs);
    font-size: 16px;
  }
  
  .empty-hint {
    font-size: 13px;
  }
  
  .job-item {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: var(--space-md);
    margin-bottom: var(--space-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }
  
  .job-item:hover {
    border-color: var(--text-muted);
    transform: translateY(-1px);
  }
  
  .job-item.selected {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 1px var(--accent-primary);
  }
  
  .job-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }
  
  .job-status {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 11px;
    font-weight: bold;
    font-family: monospace;
  }
  
  .status-success {
    background: var(--success);
    color: white;
  }
  
  .status-error {
    background: var(--error);
    color: white;
  }
  
  .status-muted {
    background: var(--text-muted);
    color: white;
  }
  
  .status-active {
    background: var(--accent-primary);
    color: var(--bg-primary);
    animation: pulse 2s ease-in-out infinite;
  }
  
  .job-id {
    font-weight: 600;
    font-size: 14px;
    font-family: 'JetBrains Mono', monospace;
  }
  
  .job-format {
    font-size: 11px;
    color: var(--accent-primary);
    padding: 2px 8px;
    background: rgba(0, 212, 170, 0.1);
    border-radius: var(--radius-sm);
    font-weight: 500;
  }
  
  .job-time {
    margin-left: auto;
    font-size: 12px;
    color: var(--text-muted);
  }
  
  .job-info {
    margin-bottom: var(--space-sm);
  }
  
  .job-overlay {
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
  
  .job-progress {
    margin-bottom: var(--space-sm);
  }
  
  .progress-stage {
    font-size: 12px;
    color: var(--text-secondary);
    display: block;
    margin-bottom: var(--space-xs);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .progress-bar {
    height: 6px;
    background: var(--bg-secondary);
    border-radius: var(--radius-full);
    overflow: hidden;
    margin-bottom: 4px;
  }
  
  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent-secondary), var(--accent-primary));
    border-radius: var(--radius-full);
    transition: width 0.3s ease;
  }
  
  .progress-percent {
    font-size: 11px;
    color: var(--text-muted);
    font-family: 'JetBrains Mono', monospace;
  }
  
  .job-actions {
    display: flex;
    gap: var(--space-sm);
  }
  
  .btn-small {
    padding: 6px 14px;
    font-size: 12px;
    border-radius: var(--radius-sm);
    font-weight: 500;
    transition: all var(--transition-fast);
  }
  
  .btn-success {
    background: var(--success);
    color: white;
  }
  
  .btn-success:hover {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }
  
  .btn-danger {
    background: transparent;
    color: var(--error);
    border: 1px solid var(--error);
  }
  
  .btn-danger:hover {
    background: var(--error);
    color: white;
  }
  
  .failed-label {
    font-size: 11px;
    color: var(--error);
    font-weight: 500;
  }
  
  .job-error {
    margin-top: var(--space-sm);
    padding: var(--space-sm);
    background: rgba(248, 113, 113, 0.1);
    border-radius: var(--radius-sm);
    color: var(--error);
    font-size: 11px;
    font-family: monospace;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }
</style>
