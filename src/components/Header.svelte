<!--
  1a. Header.svelte - top bar with logo
  1b. shows app name and maybe some quick actions later
  1c. kept it clean and minimal
-->

<script>
  import { dependencies } from '../stores/app.js';
  
  // could add menu stuff here later
</script>

<header class="header">
  <div class="logo">
    <div class="logo-icon">
      <!-- custom godz icon - two stacked rectangles representing split screen -->
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="8" rx="2" fill="var(--accent-primary)" stroke="none" opacity="0.9"/>
        <rect x="3" y="13" width="18" height="8" rx="2" fill="var(--text-secondary)" stroke="none" opacity="0.5"/>
        <path d="M12 3v18" stroke="var(--bg-primary)" stroke-width="1"/>
      </svg>
    </div>
    <span class="logo-text">GODZ</span>
    <span class="version">v1.0</span>
  </div>
  
  <div class="header-center">
    {#if $dependencies.gpuEncoder && $dependencies.gpuEncoder !== 'CPU' && $dependencies.gpuEncoder !== 'unknown'}
      <div class="gpu-badge">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
        </svg>
        GPU
      </div>
    {/if}
  </div>
  
  <div class="header-actions">
    <a href="https://github.com/Jamestownkid/Johnthebop" target="_blank" class="icon-btn" title="View on GitHub">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
      </svg>
    </a>
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    height: 52px;
    flex-shrink: 0;
  }
  
  .logo {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }
  
  .logo-icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .logo-text {
    font-weight: 700;
    font-size: 20px;
    letter-spacing: 1px;
    background: linear-gradient(135deg, var(--accent-primary), var(--text-primary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  .version {
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-weight: 500;
  }
  
  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }
  
  .gpu-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    color: var(--accent-primary);
    background: rgba(0, 255, 213, 0.1);
    padding: 4px 10px;
    border-radius: var(--radius-full);
    border: 1px solid rgba(0, 255, 213, 0.2);
  }
  
  .header-actions {
    display: flex;
    gap: var(--space-sm);
  }
  
  .icon-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
    text-decoration: none;
  }
  
  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
</style>
