<!--
  1a. InputPanel.svelte - where users set up their jobs
  1b. two modes: youtube links OR local files (fallback)
  1c. also has overlay position and dimension settings now
  took forever to get the layout right tbh
-->

<script>
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/api/dialog';
  import { invoke } from '@tauri-apps/api/tauri';
  import { startJob } from '../stores/jobs.js';
  import { dependencies, showError } from '../stores/app.js';
  
  // 2a. input mode - youtube or local files
  let inputMode = 'youtube';
  
  // 2b. youtube mode state
  let youtubeLinks = '';
  
  // 2c. local mode state
  let localBrollFiles = [];
  
  // 2d. shared form state
  let userVideoPath = '';
  let sfxFolderPath = '';
  let outputFormat = 'youtube';
  let isSubmitting = false;
  
  // 2e. overlay settings - the new hotness
  let overlayPosition = 'top';
  let splitRatio = 50;  // percentage
  let pipScale = 30;    // percentage for pip modes
  
  // 2f. custom dimensions (when format is custom)
  let customWidth = 1920;
  let customHeight = 1080;
  
  // 2g. drag and drop
  let isDragging = false;
  
  // 3a. overlay position options - loaded from backend
  let overlayOptions = [];
  let formatOptions = [];
  
  onMount(async () => {
    // get options from backend
    try {
      overlayOptions = await invoke('get_overlay_positions');
      formatOptions = await invoke('get_output_formats');
    } catch (err) {
      console.error('failed to load options:', err);
      // fallback defaults
      overlayOptions = [
        { value: 'top', label: 'B-Roll on Top', description: 'Classic split' },
        { value: 'bottom', label: 'B-Roll on Bottom', description: 'You on top' },
      ];
    }
  });
  
  // 3b. parse youtube links from textarea
  function parseLinks(text) {
    return text
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0 && (
        line.includes('youtube.com') || 
        line.includes('youtu.be')
      ));
  }
  
  // 3c. validation
  $: linksValid = inputMode === 'youtube' 
    ? parseLinks(youtubeLinks).length > 0
    : localBrollFiles.length > 0;
  
  $: videoValid = userVideoPath.length > 0;
  
  // 3d. show split ratio slider only for split modes
  $: showSplitRatio = ['top', 'bottom', 'side-by-side'].includes(overlayPosition);
  
  // 3e. show pip scale for corner modes
  $: showPipScale = ['top-left', 'top-right', 'bottom-left', 'bottom-right'].includes(overlayPosition);
  
  // 3f. show custom dims when custom format selected
  $: showCustomDims = outputFormat === 'custom';
  
  // 3g. can we submit?
  $: canSubmit = linksValid && videoValid && !isSubmitting && $dependencies.ffmpegInstalled;
  
  // 4a. file picker for user video
  async function selectUserVideo() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Video',
          extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm']
        }]
      });
      
      if (selected) {
        userVideoPath = selected;
      }
    } catch (err) {
      console.error('file picker error:', err);
    }
  }
  
  // 4b. file picker for local broll
  async function selectLocalBroll() {
    try {
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'Video',
          extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm']
        }]
      });
      
      if (selected && selected.length > 0) {
        localBrollFiles = Array.isArray(selected) ? selected : [selected];
      }
    } catch (err) {
      console.error('file picker error:', err);
    }
  }
  
  // 4c. remove a local file
  function removeLocalFile(index) {
    localBrollFiles = localBrollFiles.filter((_, i) => i !== index);
  }
  
  // 4d. folder picker for sfx
  async function selectSfxFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      
      if (selected) {
        sfxFolderPath = selected;
      }
    } catch (err) {
      console.error('folder picker error:', err);
    }
  }
  
  // 5a. drag and drop handlers
  function handleDragOver(event) {
    isDragging = true;
  }
  
  function handleDragLeave(event) {
    isDragging = false;
  }
  
  async function handleDrop(event) {
    isDragging = false;
    const text = event.dataTransfer.getData('text/plain');
    if (text && (text.includes('youtube.com') || text.includes('youtu.be'))) {
      inputMode = 'youtube';
      youtubeLinks = youtubeLinks ? youtubeLinks + '\n' + text : text;
    }
  }
  
  // 6a. submit the job
  async function handleSubmit() {
    if (!canSubmit) return;
    
    isSubmitting = true;
    
    const config = {
      userVideoPath: userVideoPath,
      outputFormat: outputFormat,
      overlayPosition: overlayPosition,
      splitRatio: splitRatio / 100,  // convert percentage to 0-1
      pipScale: pipScale / 100,
      sfxFolder: sfxFolderPath || null,
    };
    
    // add custom dimensions if needed
    if (outputFormat === 'custom') {
      config.customWidth = customWidth;
      config.customHeight = customHeight;
    }
    
    // set broll source based on mode
    if (inputMode === 'youtube') {
      config.youtubeLinks = parseLinks(youtubeLinks);
      config.localBrollPaths = null;
    } else {
      config.youtubeLinks = [];
      config.localBrollPaths = localBrollFiles;
    }
    
    const result = await startJob(config);
    
    if (result.success) {
      // clear form on success
      youtubeLinks = '';
      localBrollFiles = [];
    } else {
      showError(`failed to start job: ${result.error}`);
    }
    
    isSubmitting = false;
  }
  
  function getFilename(path) {
    return path.split('/').pop().split('\\').pop();
  }
  
  // get format info for display
  function getFormatDimensions(format) {
    const f = formatOptions.find(o => o.value === format);
    if (format === 'custom') return `${customWidth}x${customHeight}`;
    return f ? `${f.width}x${f.height}` : '';
  }
</script>

<div class="input-panel-content">
  <div class="panel-header">
    <h2>Create New Video</h2>
  </div>
  
  {#if $dependencies.checked && !$dependencies.ffmpegInstalled}
    <div class="warning-banner">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
        <line x1="12" y1="9" x2="12" y2="13"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
      <div class="warning-text">
        <strong>FFmpeg Required</strong>
        <span>Install with: sudo apt install ffmpeg</span>
      </div>
    </div>
  {/if}
  
  {#if $dependencies.checked && !$dependencies.ytdlpInstalled}
    <div class="info-banner">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="16" x2="12" y2="12"/>
        <line x1="12" y1="8" x2="12.01" y2="8"/>
      </svg>
      <span>yt-dlp not found - YouTube downloads wont work, but Local Files mode will</span>
    </div>
  {/if}
  
  <form on:submit|preventDefault={handleSubmit} class="job-form">
    <!-- B-Roll Source Section -->
    <div class="form-section">
      <h3>B-Roll Source</h3>
      
      <div class="mode-tabs">
        <button 
          type="button"
          class="mode-tab"
          class:active={inputMode === 'youtube'}
          on:click={() => inputMode = 'youtube'}
          disabled={!$dependencies.ytdlpInstalled}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19.615 3.184c-3.604-.246-11.631-.245-15.23 0-3.897.266-4.356 2.62-4.385 8.816.029 6.185.484 8.549 4.385 8.816 3.6.245 11.626.246 15.23 0 3.897-.266 4.356-2.62 4.385-8.816-.029-6.185-.484-8.549-4.385-8.816zm-10.615 12.816v-8l8 3.993-8 4.007z"/>
          </svg>
          YouTube Links
        </button>
        <button 
          type="button"
          class="mode-tab"
          class:active={inputMode === 'local'}
          on:click={() => inputMode = 'local'}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
          Local Files
        </button>
      </div>
      
      {#if inputMode === 'youtube'}
        <div 
          class="form-group drop-zone"
          class:dragging={isDragging}
          on:dragover|preventDefault={handleDragOver}
          on:dragleave={handleDragLeave}
          on:drop|preventDefault={handleDrop}
        >
          <label for="youtube-links">
            Paste YouTube Links
            <span class="label-hint">(one per line)</span>
          </label>
          <textarea
            id="youtube-links"
            bind:value={youtubeLinks}
            placeholder="https://youtube.com/watch?v=...
https://youtu.be/...
https://youtube.com/shorts/..."
            rows="4"
          ></textarea>
          {#if youtubeLinks && !linksValid}
            <span class="validation-error">no valid youtube links found</span>
          {:else if linksValid}
            <span class="validation-success">{parseLinks(youtubeLinks).length} links ready</span>
          {/if}
        </div>
      {:else}
        <div class="form-group">
          <label>Select Your B-Roll Videos</label>
          <div class="local-files-area">
            <button type="button" class="btn-secondary btn-add-files" on:click={selectLocalBroll}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              Add Video Files
            </button>
            
            {#if localBrollFiles.length > 0}
              <div class="file-list">
                {#each localBrollFiles as file, i}
                  <div class="file-item">
                    <span class="file-name">{getFilename(file)}</span>
                    <button type="button" class="btn-remove" on:click={() => removeLocalFile(i)}>x</button>
                  </div>
                {/each}
              </div>
              <span class="validation-success">{localBrollFiles.length} files ready</span>
            {:else}
              <p class="empty-hint">no files selected - click above to add videos</p>
            {/if}
          </div>
        </div>
      {/if}
    </div>
    
    <!-- Your Video Section -->
    <div class="form-section">
      <h3>Your Video</h3>
      <div class="form-group">
        <label for="user-video">Select your talking head video</label>
        <div class="file-picker">
          <input id="user-video" type="text" readonly value={userVideoPath} placeholder="Click Browse to select..." />
          <button type="button" class="btn-secondary" on:click={selectUserVideo}>Browse</button>
        </div>
      </div>
    </div>
    
    <!-- Output Settings Section -->
    <div class="form-section">
      <h3>Output Settings</h3>
      
      <!-- Output Format -->
      <div class="form-group">
        <label>Output Format</label>
        <div class="format-options">
          {#each formatOptions as format}
            <label class="format-option" class:selected={outputFormat === format.value}>
              <input type="radio" bind:group={outputFormat} value={format.value} />
              <span class="format-label">{format.label}</span>
              <span class="format-dims">
                {format.value === 'custom' ? 'Custom' : `${format.width}x${format.height}`}
              </span>
            </label>
          {/each}
        </div>
      </div>
      
      <!-- Custom Dimensions -->
      {#if showCustomDims}
        <div class="form-group dims-group">
          <label>Custom Dimensions</label>
          <div class="dims-inputs">
            <input type="number" bind:value={customWidth} min="320" max="4096" placeholder="Width" />
            <span class="dims-x">x</span>
            <input type="number" bind:value={customHeight} min="240" max="4096" placeholder="Height" />
            <span class="dims-hint">pixels</span>
          </div>
        </div>
      {/if}
      
      <!-- Overlay Position -->
      <div class="form-group">
        <label>B-Roll Position</label>
        <select bind:value={overlayPosition} class="overlay-select">
          {#each overlayOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
        {#if overlayOptions.find(o => o.value === overlayPosition)}
          <span class="option-description">
            {overlayOptions.find(o => o.value === overlayPosition).description}
          </span>
        {/if}
      </div>
      
      <!-- Split Ratio Slider -->
      {#if showSplitRatio}
        <div class="form-group slider-group">
          <label>Split Ratio: {splitRatio}% B-Roll / {100 - splitRatio}% You</label>
          <input 
            type="range" 
            bind:value={splitRatio} 
            min="20" 
            max="80" 
            step="5"
            class="slider"
          />
          <div class="slider-labels">
            <span>More You</span>
            <span>More B-Roll</span>
          </div>
        </div>
      {/if}
      
      <!-- PiP Scale Slider -->
      {#if showPipScale}
        <div class="form-group slider-group">
          <label>Overlay Size: {pipScale}% of screen</label>
          <input 
            type="range" 
            bind:value={pipScale} 
            min="15" 
            max="50" 
            step="5"
            class="slider"
          />
          <div class="slider-labels">
            <span>Smaller</span>
            <span>Larger</span>
          </div>
        </div>
      {/if}
    </div>
    
    <!-- SFX Section (Optional) -->
    <div class="form-section collapsible">
      <h3>Sound Effects <span class="optional">(optional)</span></h3>
      <div class="form-group">
        <label for="sfx-folder">Folder with sound effect files</label>
        <div class="file-picker">
          <input id="sfx-folder" type="text" readonly value={sfxFolderPath} placeholder="Select folder..." />
          <button type="button" class="btn-secondary" on:click={selectSfxFolder}>Browse</button>
          {#if sfxFolderPath}
            <button type="button" class="btn-clear" on:click={() => sfxFolderPath = ''}>x</button>
          {/if}
        </div>
      </div>
    </div>
    
    <!-- Submit Button -->
    <div class="form-actions">
      <button type="submit" class="btn-primary btn-large" disabled={!canSubmit}>
        {#if isSubmitting}
          <span class="spinner"></span>
          Starting...
        {:else}
          Create Video
        {/if}
      </button>
      {#if !$dependencies.ffmpegInstalled}
        <p class="submit-hint">Install FFmpeg first to create videos</p>
      {:else if !linksValid}
        <p class="submit-hint">Add some B-Roll to get started</p>
      {:else if !videoValid}
        <p class="submit-hint">Select your video file</p>
      {/if}
    </div>
  </form>
</div>

<style>
  .input-panel-content { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
  .panel-header { padding: var(--space-md); border-bottom: 1px solid var(--border-color); flex-shrink: 0; }
  .panel-header h2 { font-size: 18px; font-weight: 600; color: var(--accent-primary); }
  
  .warning-banner { display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-md); background: rgba(248, 113, 113, 0.1); border-bottom: 1px solid var(--error); color: var(--error); }
  .warning-text { display: flex; flex-direction: column; gap: 2px; font-size: 13px; }
  
  .info-banner { display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-sm) var(--space-md); background: rgba(96, 165, 250, 0.1); border-bottom: 1px solid var(--info); color: var(--info); font-size: 12px; }
  
  .job-form { flex: 1; overflow-y: auto; padding: var(--space-md); display: flex; flex-direction: column; gap: var(--space-md); }
  
  .form-section { background: var(--bg-tertiary); border-radius: var(--radius-md); padding: var(--space-md); display: flex; flex-direction: column; gap: var(--space-md); }
  .form-section h3 { font-size: 14px; font-weight: 600; color: var(--text-secondary); margin-bottom: var(--space-xs); }
  .form-section h3 .optional { font-weight: 400; color: var(--text-muted); font-size: 12px; }
  
  .form-group { display: flex; flex-direction: column; gap: var(--space-xs); }
  .form-group label { font-weight: 500; font-size: 13px; display: flex; align-items: center; gap: var(--space-sm); }
  .label-hint { color: var(--text-muted); font-weight: 400; font-size: 12px; }
  
  .mode-tabs { display: flex; gap: 2px; background: var(--bg-secondary); padding: 3px; border-radius: var(--radius-md); }
  .mode-tab { flex: 1; display: flex; align-items: center; justify-content: center; gap: var(--space-sm); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); font-size: 13px; font-weight: 500; color: var(--text-secondary); transition: all var(--transition-fast); }
  .mode-tab:hover:not(:disabled) { color: var(--text-primary); background: var(--bg-hover); }
  .mode-tab.active { background: var(--accent-primary); color: var(--bg-primary); }
  .mode-tab:disabled { opacity: 0.5; cursor: not-allowed; }
  
  .drop-zone { transition: all var(--transition-fast); }
  .drop-zone.dragging { background: rgba(0, 212, 170, 0.1); border-radius: var(--radius-md); outline: 2px dashed var(--accent-primary); outline-offset: -2px; padding: var(--space-sm); }
  
  .form-group textarea { font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 12px; line-height: 1.5; resize: none; }
  
  .validation-error { color: var(--error); font-size: 12px; }
  .validation-success { color: var(--success); font-size: 12px; }
  
  .local-files-area { display: flex; flex-direction: column; gap: var(--space-sm); }
  .btn-add-files { display: flex; align-items: center; justify-content: center; gap: var(--space-sm); padding: var(--space-md); border-style: dashed; }
  
  .file-list { display: flex; flex-direction: column; gap: 4px; max-height: 120px; overflow-y: auto; }
  .file-item { display: flex; align-items: center; justify-content: space-between; padding: var(--space-sm) var(--space-md); background: var(--bg-secondary); border-radius: var(--radius-sm); font-size: 12px; }
  .file-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
  .btn-remove { width: 20px; height: 20px; display: flex; align-items: center; justify-content: center; border-radius: var(--radius-sm); color: var(--text-muted); font-size: 14px; flex-shrink: 0; }
  .btn-remove:hover { background: var(--error); color: white; }
  
  .empty-hint { color: var(--text-muted); font-size: 12px; text-align: center; padding: var(--space-sm); }
  
  .file-picker { display: flex; gap: var(--space-sm); }
  .file-picker input { flex: 1; cursor: pointer; font-size: 12px; }
  .btn-clear { width: 36px; background: var(--bg-secondary); border: 1px solid var(--border-color); border-radius: var(--radius-md); color: var(--text-secondary); font-size: 16px; }
  .btn-clear:hover { background: var(--error); color: white; border-color: var(--error); }
  
  .format-options { display: grid; grid-template-columns: repeat(4, 1fr); gap: var(--space-sm); }
  .format-option { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: var(--space-sm); background: var(--bg-secondary); border: 2px solid var(--border-color); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-fast); text-align: center; }
  .format-option input { display: none; }
  .format-option:hover { border-color: var(--text-muted); }
  .format-option.selected { border-color: var(--accent-primary); background: rgba(0, 212, 170, 0.1); }
  .format-label { font-weight: 500; font-size: 12px; }
  .format-dims { color: var(--text-muted); font-size: 10px; }
  
  .overlay-select { width: 100%; padding: var(--space-sm) var(--space-md); font-size: 13px; background: var(--bg-secondary); border: 1px solid var(--border-color); border-radius: var(--radius-md); color: var(--text-primary); cursor: pointer; }
  .overlay-select:focus { border-color: var(--accent-primary); outline: none; }
  .option-description { font-size: 11px; color: var(--text-muted); }
  
  .dims-group .dims-inputs { display: flex; align-items: center; gap: var(--space-sm); }
  .dims-inputs input { width: 100px; text-align: center; }
  .dims-x { color: var(--text-muted); font-weight: 500; }
  .dims-hint { color: var(--text-muted); font-size: 11px; }
  
  .slider-group { }
  .slider { width: 100%; height: 6px; border-radius: var(--radius-full); appearance: none; background: var(--bg-secondary); cursor: pointer; }
  .slider::-webkit-slider-thumb { appearance: none; width: 18px; height: 18px; border-radius: 50%; background: var(--accent-primary); cursor: pointer; border: 2px solid var(--bg-primary); }
  .slider-labels { display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); }
  
  .form-actions { margin-top: auto; padding-top: var(--space-md); border-top: 1px solid var(--border-color); }
  .btn-large { width: 100%; padding: var(--space-md); font-size: 15px; display: flex; align-items: center; justify-content: center; gap: var(--space-sm); }
  .submit-hint { text-align: center; font-size: 12px; color: var(--text-muted); margin-top: var(--space-sm); }
  
  .spinner { width: 16px; height: 16px; border: 2px solid var(--bg-primary); border-top-color: transparent; border-radius: 50%; animation: spin 1s linear infinite; }
  
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
