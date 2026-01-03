// vite.config.js - bundler config
// vite is way faster than webpack, trust

import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  
  // tauri needs this for the dev server
  clearScreen: false,
  
  server: {
    port: 5173,
    strictPort: true,
  },
  
  envPrefix: ['VITE_', 'TAURI_'],
  
  build: {
    // tauri supports es2021
    target: ['es2021', 'chrome100', 'safari13'],
    // dont minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
