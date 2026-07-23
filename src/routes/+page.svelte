<!-- src/routes/+page.svelte -->
<script>
  import { onMount } from 'svelte'
  import { goto } from '$app/navigation'
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
  import { GENERAL_CATEGORIES } from '$lib/perfData.js'
  import { specsStore, specsErrorStore, generalStore } from '$lib/loadStore.js'
  import logo from '../assets/logo.png'

  // This screen never resizes itself and never observes its own content —
  // it's a fixed box that just sits there until everything is ready, then
  // hands off to mainDash (which owns all the dynamic auto-fit logic).
  const WIDTH = 340
  const HEIGHT = 300

  async function loadEverything() {
    let specs = null

    try {
      specs = await invoke('get_system_specs')
      specsStore.set(specs)
    } catch (e) {
      specsErrorStore.set(String(e))
    }

    if (specs) {
      generalStore.set({ stage: 'loading', results: {}, notes: {}, error: false, errorDetail: '' })
      try {
        const resp = await invoke('benchmark_apps', { specs, dept: null, apps: GENERAL_CATEGORIES })
        const isFallback = resp.estimator === 'fallback'
        if (isFallback) {
          generalStore.set({ stage: 'done', results: {}, notes: {}, error: true, errorDetail: resp.error_detail || '' })
        } else {
          const results = {}, notes = {}
          GENERAL_CATEGORIES.forEach(c => {
            const entry = resp.results[c]
            results[c] = entry ? entry.score : 0
            notes[c] = entry ? entry.verdict : ''
          })
          generalStore.set({ stage: 'done', results, notes, error: false, errorDetail: '' })
        }
      } catch (e) {
        generalStore.set({ stage: 'done', results: {}, notes: {}, error: true, errorDetail: String(e) })
      }
    }

    // Whether things succeeded or failed, we're done loading — mainDash
    // knows how to render either the results or the error states.
    goto('/mainDash')
  }

  onMount(() => {
    // Set the window to a fixed, comfortable size once and never touch it
    // again from this page — no ResizeObserver, no re-measuring, no
    // content-driven scaling. mainDash takes over sizing after nav.
    getCurrentWindow()
      .setSize(new LogicalSize(WIDTH, HEIGHT))
      .catch(() => {
        // Not running inside Tauri (e.g. `npm run dev` in a browser tab) — ignore.
      })

    loadEverything()
  })
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link
    href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<main>
  <div class="logo-wrap">
    <div class="spinner" aria-hidden="true"></div>
    <img class="logo" src={logo} alt="MakeDo logo" />
  </div>
</main>

<style>
  :global(:root) {
    --bg: #ffffff;
    --accent: #ff6b00;
    --border: #e0e0e0;
    --font-mono: 'JetBrains Mono', monospace;
  }
  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }
  :global(html, body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: var(--bg);
  }

  main {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
  }

  .logo-wrap {
    position: relative;
    width: 96px;
    height: 96px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo {
    width: 56px;
    height: 56px;
    border-radius: 10px;
    display: block;
    position: relative;
    z-index: 1;
  }

  .spinner {
    position: absolute;
    inset: 0;
    width: 96px;
    height: 96px;
    border: 4px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>