# Icons TODO

yo cursor, need you to make the app icons for this project

## what we need

the app needs icons in these sizes/formats for tauri:
- 32x32.png
- 128x128.png  
- 128x128@2x.png (256x256)
- icon.icns (macOS bundle)
- icon.ico (Windows)

## design brief

keep it simple, we're going for a clean look:
- dark background (#121212 or transparent)
- accent color is teal/cyan (#00d4aa)
- should represent "video editing" or "scrambling"

ideas:
- could be film strip pieces being mixed
- could be two rectangles overlapping (representing split screen)
- could be a play button with some kind of shuffle indicator
- keep it minimal, works well at small sizes

## where to put them

save the icons to:
- `/src-tauri/icons/` (for tauri build)
- `/public/` (for web/dev)
- also make a `/public/favicon.svg` for the browser tab

## tools to use

you can use:
- figma
- illustrator
- inkscape
- or just generate with code (svg is fine)

## example svg for reference

here's a rough idea of what could work:

```svg
<svg width="128" height="128" viewBox="0 0 128 128" fill="none">
  <!-- background -->
  <rect width="128" height="128" rx="24" fill="#121212"/>
  
  <!-- top video rectangle -->
  <rect x="24" y="20" width="80" height="40" rx="8" fill="#00d4aa" opacity="0.8"/>
  
  <!-- bottom video rectangle -->
  <rect x="24" y="68" width="80" height="40" rx="8" fill="#ffffff" opacity="0.9"/>
  
  <!-- shuffle/scramble indicator lines -->
  <path d="M64 56 L64 72" stroke="#00d4aa" stroke-width="4" stroke-linecap="round"/>
  <path d="M52 64 L76 64" stroke="#00d4aa" stroke-width="4" stroke-linecap="round"/>
</svg>
```

feel free to iterate on this and make it look better

## after making icons

run `npm run tauri icon` if theres a tauri icon generator
or manually resize and place files

thanks homie
