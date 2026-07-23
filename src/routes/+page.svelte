<script>
  import { onMount, onDestroy } from 'svelte'
  import { fade, fly } from 'svelte/transition'
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWindow, LogicalSize, currentMonitor } from '@tauri-apps/api/window'
  import { DEPARTMENTS, DEPT_ICONS, APP_ICONS, SPEC_ICONS, UI_ICONS, GENERAL_CATEGORIES } from '$lib/perfData.js'
  import logo from '../assets/logo.png'

  // ── specs ──
  let specs = null
  let specsError = ''
  let specsLoading = true

  // ── general readiness (auto-run once specs land) ──
  let general = { stage: 'idle', results: {}, notes: {}, error: false, errorDetail: '' } // idle|loading|done

  // ── app-specific picker ──
  let picker = { open: false, step: 'dept', dept: null, apps: [] } // step: dept|apps
  let bench = { stage: 'idle', results: {}, notes: {}, error: false, errorDetail: '' } // idle|loading|done

  // ─────────────────────────────────────────────────────────────
  // Window auto-fit: no scrollbars, ever. The window always resizes to
  // match its content — but it's also clamped to the screen's usable
  // area. If the content's natural size would be bigger than the screen
  // can fit, everything (boxes, gaps, text — the whole layout) is scaled
  // down together with a single CSS transform so it fits inside the
  // window edge to edge, instead of clipping or scrolling anything.
  // ─────────────────────────────────────────────────────────────
  let rootEl
  let mainObserver
  let fitFrame = null
  let contentScale = 1 // 1 = natural size; <1 = shrunk to fit the screen

  // Known Tauri quirk: on macOS, setSize() can land on the *outer*
  // window size instead of the inner content size, so the titlebar
  // eats into the content. A small fudge factor keeps content from
  // getting clipped there. https://github.com/tauri-apps/tauri/issues/15136
  const MAC_TITLEBAR_FUDGE = typeof navigator !== 'undefined' && /Mac/.test(navigator.userAgent) ? 28 : 0

  // Reserve a little room around the edges so the window doesn't sit
  // flush against the screen border or a taskbar/dock we don't know
  // the exact height of.
  const SCREEN_MARGIN_X = 24
  const SCREEN_MARGIN_Y = 60

  // Natural (unscaled) content size. offsetWidth/offsetHeight — unlike
  // getBoundingClientRect() — reflect the pre-transform layout box, so
  // these stay accurate no matter what contentScale is currently applied.
  function measureNatural() {
    const w = rootEl ? rootEl.offsetWidth : 720
    const h = rootEl ? rootEl.offsetHeight : 0
    return { w, h: h + MAC_TITLEBAR_FUDGE }
  }

  // Figures out how much logical space is actually available on the
  // screen the window currently sits on, so we know where "the edge
  // of the window" really is before asking for a bigger size.
  async function availableScreenSize() {
    try {
      const mon = await currentMonitor()
      if (!mon) return null
      const dpi = mon.scaleFactor || 1
      // Prefer the monitor's usable work area (excludes taskbar/dock) when
      // the platform exposes it; otherwise fall back to the full monitor size.
      const area = mon.workArea?.size || mon.size
      return {
        w: Math.floor(area.width / dpi - SCREEN_MARGIN_X),
        h: Math.floor(area.height / dpi - SCREEN_MARGIN_Y),
      }
    } catch (e) {
      return null // not running inside Tauri, or monitor info unavailable
    }
  }

  function fitWindow() {
    if (fitFrame) cancelAnimationFrame(fitFrame)
    fitFrame = requestAnimationFrame(async () => {
      const { w: naturalW, h: naturalH } = measureNatural()
      const screen = await availableScreenSize()

      // Shrink (never enlarge) just enough that the natural layout fits
      // inside whatever screen space is available.
      const fitW = screen ? Math.min(1, screen.w / naturalW) : 1
      const fitH = screen ? Math.min(1, screen.h / naturalH) : 1
      contentScale = Math.max(Math.min(fitW, fitH), 0.35) // sane floor so it never vanishes

      const targetW = Math.round(naturalW * contentScale)
      const targetH = Math.round(naturalH * contentScale)

      try {
        await getCurrentWindow().setSize(new LogicalSize(targetW, targetH))
      } catch (e) {
        // Not running inside Tauri (e.g. `npm run dev` in a browser tab) — ignore.
      }
    })

  }

  onMount(async () => {
    mainObserver = new ResizeObserver(fitWindow)
    mainObserver.observe(rootEl)
    document.fonts?.ready?.then(fitWindow)
    fitWindow()

    try {
      specs = await invoke('get_system_specs')
    } catch (e) {
      specsError = String(e)
    } finally {
      specsLoading = false
    }
    if (specs) runGeneral()
  })

  onDestroy(() => {
    mainObserver?.disconnect()
    if (fitFrame) cancelAnimationFrame(fitFrame)
  })

  async function runGeneral() {
    general = { stage: 'loading', results: {}, notes: {}, error: false, errorDetail: '' }
    try {
      const resp = await invoke('benchmark_apps', { specs, dept: null, apps: GENERAL_CATEGORIES })
      const isFallback = resp.estimator === 'fallback'
      if (isFallback) {
        general = { stage: 'done', results: {}, notes: {}, error: true, errorDetail: resp.error_detail || '' }
        return
      }
      const results = {}, notes = {}
      GENERAL_CATEGORIES.forEach(c => {
        const entry = resp.results[c]
        results[c] = entry ? entry.score : 0
        notes[c] = entry ? entry.verdict : ''
      })
      general = { stage: 'done', results, notes, error: false, errorDetail: '' }
    } catch (e) {
      console.error('general benchmark failed', e)
      general = { stage: 'done', results: {}, notes: {}, error: true, errorDetail: String(e) }
    }
  }

  function openPicker() {
    picker = { open: true, step: 'dept', dept: null, apps: [] }
    bench = { stage: 'idle', results: {}, notes: {}, error: false, errorDetail: '' }
  }
  function closePicker() {
    picker.open = false
    fitWindow()
  }
  function selectDept(d) {
    picker.step = 'apps'
    picker.dept = d
    picker.apps = []
    picker = picker
  }
  function backToDept() {
    picker.step = 'dept'
    picker.apps = []
    bench = { stage: 'idle', results: {}, notes: {}, error: false, errorDetail: '' }
    picker = picker
  }
  function toggleApp(app) {
    const i = picker.apps.indexOf(app)
    if (i === -1) picker.apps.push(app)
    else picker.apps.splice(i, 1)
    picker = picker
  }

  async function runAppBenchmark() {
    if (!picker.apps.length) return
    bench = { stage: 'loading', results: {}, notes: {}, error: false, errorDetail: '' }
    try {
      const resp = await invoke('benchmark_apps', {
        specs,
        dept: picker.dept.label,
        apps: picker.apps,
      })
      const isFallback = resp.estimator === 'fallback'
      if (isFallback) {
        bench = { stage: 'done', results: {}, notes: {}, error: true, errorDetail: resp.error_detail || '' }
        return
      }
      const results = {}, notes = {}
      picker.apps.forEach(a => {
        const entry = resp.results[a]
        results[a] = entry ? entry.score : 0
        notes[a] = entry ? entry.verdict : ''
      })
      bench = { stage: 'done', results, notes, error: false, errorDetail: '' }
    } catch (e) {
      console.error('app benchmark failed', e)
      bench = { stage: 'done', results: {}, notes: {}, error: true, errorDetail: String(e) }
    }
  }

  // ── score band color (used by the progress bars) ──
  function gaugeColor(v) {
    return v >= 66 ? 'var(--green)' : v >= 33 ? 'var(--olive)' : 'var(--red)'
  }
  // Fallback description for the rare case the estimator returns a score with no verdict text.
  function bandDescription(v) {
    if (v >= 71) return 'Smooth — handles this comfortably.'
    if (v >= 46) return 'Usable, but expect occasional slowdowns on heavier tasks.'
    if (v >= 21) return 'Workable, but expect jitters and longer render/export times.'
    return "Not really built for this — expect stutters and long waits."
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link
    href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<main bind:this={rootEl} style="transform: scale({contentScale}); transform-origin: top left;">
  <div class="brandbar">
    <img class="brand-logo" src={logo} alt="MakeDo logo" />
    <span class="brand-caption">built by <strong>makedo</strong></span>
  </div>

  <header>
    <h1>Benchy</h1>
    <p class="sub">Reads your real hardware, then estimates what it can actually handle.</p>
  </header>

  {#if specsLoading || (specs && general.stage !== 'done')}
    <div class="panel loading-panel" in:fade={{ duration: 150 }}>
      {#if specsLoading}
        <div class="pc-reader" in:fade={{ duration: 150 }}>
          <svg class="pc-icon" viewBox="0 0 64 46" width="76" height="55">
            <rect x="2" y="2" width="60" height="34" rx="3" fill="none" stroke="var(--border2)" stroke-width="2" />
            <rect x="24" y="36" width="16" height="4" fill="var(--border2)" />
            <rect x="15" y="40" width="34" height="3" rx="1.5" fill="var(--border2)" />
            <rect x="2" y="2" width="60" height="34" rx="3" fill="none" stroke="none" />
            <clipPath id="pcScreenClip"><rect x="4" y="4" width="56" height="30" rx="1" /></clipPath>
            <g clip-path="url(#pcScreenClip)">
              <rect class="scan-line" x="4" y="4" width="56" height="3" fill="var(--accent)" />
            </g>
          </svg>
        </div>
        <div class="loading-title">Reading your hardware…</div>
        <div class="loading-sub">Pulling CPU, GPU, RAM &amp; storage details</div>
      {:else}
        <div class="ai-pulse" in:fade={{ duration: 150 }}>
          <span class="spark-big">{@html UI_ICONS.spark}</span>
        </div>
        <div class="loading-title">Running performance estimation tests…</div>
        <div class="loading-sub">Scoring your hardware against real workloads</div>
      {/if}
    </div>
  {:else if specsError}
    <div class="panel error-panel">
      <span class="icon-lg">{@html UI_ICONS.alert}</span>
      <div>Couldn't read system specs.<div class="error-detail">{specsError}</div></div>
    </div>
  {:else if specs}
    <section class="panel specs-panel" in:fade={{ duration: 220 }}>
      <div class="specs-title">
        <span class="spec-icon inline">{@html SPEC_ICONS.host}</span>
        {specs.hostname} <span class="os-tag">{specs.os} {specs.os_version}</span>
      </div>
      <div class="specs-grid">
        <div class="spec-card">
          <div class="spec-label"><span class="spec-icon">{@html SPEC_ICONS.cpu}</span>CPU</div>
          <div class="spec-value">{specs.cpu_brand}</div>
          <div class="spec-sub">{specs.cpu_cores} cores / {specs.cpu_threads} threads</div>
        </div>
        <div class="spec-card">
          <div class="spec-label"><span class="spec-icon">{@html SPEC_ICONS.ram}</span>RAM</div>
          <div class="spec-value">{specs.ram_total_gb} GB</div>
          <div class="spec-sub">{specs.ram_available_gb} GB available now</div>
        </div>
        <div class="spec-card">
          <div class="spec-label"><span class="spec-icon">{@html SPEC_ICONS.gpu}</span>GPU</div>
          <div class="spec-value">{specs.gpu_name}</div>
        </div>
        <div class="spec-card">
          <div class="spec-label"><span class="spec-icon">{@html SPEC_ICONS.storage}</span>Storage</div>
          {#each specs.disks.slice(0, 2) as d}
            <div class="spec-value small">{d.name || 'Disk'}: {d.total_gb} GB ({d.kind})</div>
          {/each}
        </div>
      </div>
    </section>

    <section class="panel general-panel">
      <div class="panel-header">
        <span class="panel-title"><span class="spark">{@html UI_ICONS.spark}</span>General readiness</span>
      </div>

      {#if general.error}
        <div class="unavailable" in:fade={{ duration: 200 }}>
          <span class="icon-lg">{@html UI_ICONS.alert}</span>
          <div>
            <div class="unavailable-title">Performance estimator unavailable</div>
            <div class="unavailable-sub">
              {general.errorDetail || "Couldn't reach the estimator, so no scores are shown."}
            </div>
          </div>
        </div>
      {:else}
        <div class="bar-list">
          {#each GENERAL_CATEGORIES as cat, i}
            {@const val = general.results[cat] ?? 0}
            <div class="bar-row" in:fly={{ y: 6, duration: 220, delay: i * 40 }}>
              <div class="bar-top">
                <span class="bar-name">{cat}</span>
                <span class="bar-pct" style="color:{gaugeColor(val)}">{val}%</span>
              </div>
              <div class="bar-track">
                <div class="bar-fill" style="width:{val}%; background:{gaugeColor(val)}"></div>
              </div>
              <div class="bar-note">{general.notes[cat] || bandDescription(val)}</div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <section class="panel cta-panel">
      <div>
        <div class="cta-title">Want numbers for the exact apps you use?</div>
        <div class="cta-sub">Pick your line of work and we'll score it app by app.</div>
      </div>
      <button class="primary-btn" on:click={openPicker}>Get app-specific benchmark →</button>
    </section>
  {/if}


  {#if picker.open}
    <div class="modal-backdrop" on:click={closePicker} transition:fade={{ duration: 150 }}>
      <div class="modal" on:click|stopPropagation transition:fly={{ y: 12, duration: 200 }}>
        <div class="modal-header">
          <span>App-specific benchmark</span>
          <button class="close-btn" on:click={closePicker} aria-label="Close">✕</button>
        </div>

        {#if picker.step === 'dept'}
          <div class="modal-sub">What do you use this machine for?</div>
          <div class="dept-grid">
            {#each DEPARTMENTS as d}
              <button class="dept-btn" on:click={() => selectDept(d)}>
                <span class="dept-icon">{@html DEPT_ICONS[d.icon] || ''}</span>
                {d.label}
              </button>
            {/each}
          </div>
        {:else if bench.stage === 'idle'}
          <div class="modal-topbar">
            <button class="chip-btn" on:click={backToDept}>
              <span class="dept-icon">{@html DEPT_ICONS[picker.dept.icon] || ''}</span>
              {picker.dept.label}
            </button>
            <button class="run-btn" disabled={!picker.apps.length} on:click={runAppBenchmark}>
              Run test →
            </button>
          </div>
          <div class="modal-sub">Apps you use</div>
          <div class="app-grid">
            {#each picker.dept.apps as a}
              <button
                class="app-pill {picker.apps.includes(a) ? 'active' : ''}"
                on:click={() => toggleApp(a)}
              >
                {#if APP_ICONS[a]}
                  <img class="app-icon" src={APP_ICONS[a]} alt="" />
                {/if}
                {a}
              </button>
            {/each}
          </div>
        {:else if bench.stage === 'loading'}
          <div class="panel loading-panel loading-panel-modal" in:fade={{ duration: 150 }}>
            <div class="ai-pulse">
              <span class="spark-big">{@html UI_ICONS.spark}</span>
            </div>
            <div class="loading-title">Running performance estimation tests…</div>
            <div class="loading-sub">Scoring {picker.apps.join(', ')}</div>
          </div>
        {:else if bench.error}
          <div class="unavailable" in:fade={{ duration: 200 }}>
            <span class="icon-lg">{@html UI_ICONS.alert}</span>
            <div>
              <div class="unavailable-title">Performance estimator unavailable</div>
              <div class="unavailable-sub">
                {bench.errorDetail || "Couldn't reach the estimator, so no scores are shown."}
              </div>
            </div>
          </div>
          <button class="text-btn" on:click={backToDept}>← try different apps</button>
        {:else}
          <div class="bar-list">
            {#each picker.apps as a, i}
              {@const val = bench.results[a] ?? 0}
              <div class="bar-row" in:fly={{ y: 6, duration: 220, delay: i * 40 }}>
                <div class="bar-top">
                  <span class="bar-name">{a}</span>
                  <span class="bar-pct" style="color:{gaugeColor(val)}">{val}%</span>
                </div>
                <div class="bar-track">
                  <div class="bar-fill" style="width:{val}%; background:{gaugeColor(val)}"></div>
                </div>
                <div class="bar-note">{bench.notes[a] || bandDescription(val)}</div>
              </div>
            {/each}
          </div>
          <button class="text-btn" on:click={backToDept}>← try different apps</button>
        {/if}
      </div>
    </div>
  {/if}
</main>

<style>
  /* ── design tokens, lifted straight from the MakeDo site ── */
  :global(:root) {
    --bg: #ffffff;
    --bg2: #f5f5f5;
    --bg3: #efefef;
    --bg4: #e8e8e8;
    --accent: #ff6b00;
    --accent2: #ff9040;
    --green: #16a34a;
    --lightgreen: #86efac;
    --olive: #65a30d;
    --red: #ef4444;
    --text: #111111;
    --text2: #555555;
    --text3: #999999;
    --border: #e0e0e0;
    --border2: #aaaaaa;
    --r2: 2px;
    --r4: 4px;
    --r6: 6px;
    --font-mono: 'JetBrains Mono', monospace;
    --font-display: 'Space Grotesk', sans-serif;
  }
  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }
  :global(html, body) {
    margin: 0;
    padding: 0;
    overflow: hidden; /* the window resizes to content instead of scrolling */
    background: var(--bg);
  }
  :global(body) {
    font-family: var(--font-display);
    color: var(--text);
    background-image: radial-gradient(circle, rgba(0, 0, 0, 0.08) 1px, transparent 1px);
    background-size: 24px 24px;
  }

  main {
    width: 720px;
    box-sizing: border-box;
    padding: 26px 24px 28px;
  }

  /* ── brand bar ── */
  .brandbar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 20px;
  }
  .brand-logo {
    width: 22px;
    height: 22px;
    border-radius: var(--r4);
    display: block;
  }
  .brand-caption {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text3);
  }
  .brand-caption strong {
    color: var(--accent);
    font-weight: 600;
  }

  header h1 {
    font-family: var(--font-mono);
    font-size: 22px;
    font-weight: 700;
    margin: 0 0 4px;
    letter-spacing: -0.01em;
    color: var(--text);
  }
  .sub {
    color: var(--text2);
    font-size: 12px;
    font-weight: 400;
    margin: 0 0 20px;
  }

  .panel {
    background: #fff;
    border: 1px solid var(--border);
    border-radius: var(--r4);
    padding: 18px 20px;
    margin-bottom: 14px;
  }

  /* ── unified loading screen: nothing about specs/scores shows until the
     estimation actually finishes. Two phases share the same shell.
     Fixed to a generous height that roughly matches the eventual
     specs+results content, so there's nothing else needed to hold the
     window at a sane size while this is the only thing on screen — it
     doesn't shrink to a tiny box and then jump/resize once real content
     replaces it. ── */
  .loading-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 40px 20px 34px;
    min-height: 460px;
    /* Hardcoded to the actual Tauri window width (525px in tauri.conf.json)
       minus main's own left+right padding (24px + 24px), so this card never
       renders wider than the real window — regardless of main's 720px
       layout width or the runtime content-scale transform. */
    width: 150px;
    max-width: 100%;
    box-sizing: border-box;
    margin-left: auto;
    margin-right: auto;
  }
  .loading-panel-modal {
    min-height: 220px;
    padding: 34px 20px 28px;
  }
  .loading-title {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--text);
    margin-top: 16px;
  }
  .loading-sub {
    font-size: 11px;
    color: var(--text3);
    margin-top: 5px;
    max-width: 280px;
    box-sizing: border-box;
  }

  /* phase 1 — reading hardware: a little monitor with a scanning beam */
  .pc-reader {
    display: flex;
  }
  .scan-line {
    animation: scan-sweep 1.3s ease-in-out infinite;
  }
  @keyframes scan-sweep {
    0% {
      transform: translateY(0);
      opacity: 0.4;
    }
    50% {
      transform: translateY(27px);
      opacity: 1;
    }
    100% {
      transform: translateY(0);
      opacity: 0.4;
    }
  }

  /* phase 2 — running the performance estimation: a pulsing spark */
  .ai-pulse {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent);
    animation: ai-pulse-beat 1.1s ease-in-out infinite;
  }
  .ai-pulse :global(svg) {
    width: 100%;
    height: 100%;
  }
  @keyframes ai-pulse-beat {
    0%, 100% {
      transform: scale(0.88);
      opacity: 0.6;
    }
    50% {
      transform: scale(1.05);
      opacity: 1;
    }
  }
  .spark-big {
    display: block;
    width: 100%;
    height: 100%;
  }

  .error-panel {
    text-align: center;
    font-size: 12px;
    padding: 30px 10px;
    color: var(--red);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .error-detail {
    color: var(--text3);
    font-size: 10.5px;
    font-family: var(--font-mono);
    margin-top: 4px;
  }

  /* ── unavailable state (replaces made-up numbers) — a quiet note, not an alert ── */
  .unavailable {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: var(--r2);
    padding: 14px 16px;
  }
  .unavailable-title {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--text);
    margin-bottom: 3px;
  }
  .unavailable-sub {
    font-size: 10.5px;
    color: var(--text3);
    line-height: 1.5;
  }
  .icon-lg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    color: var(--accent);
  }
  .icon-lg :global(svg) {
    width: 100%;
    height: 100%;
  }

  .specs-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 700;
    color: var(--text);
    margin-bottom: 14px;
  }
  .spec-icon.inline {
    color: var(--accent);
  }
  .os-tag {
    font-weight: 400;
    color: var(--text3);
    font-size: 11px;
    margin-left: 4px;
  }
  .specs-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }
  .spec-card {
    background: #fff;
    border: 1px solid var(--border);
    border-radius: var(--r2);
    padding: 12px 14px;
    transition: border-color 0.15s;
  }
  .spec-card:hover {
    border-color: var(--border2);
  }
  .spec-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text3);
    margin-bottom: 6px;
  }
  .spec-icon {
    width: 14px;
    height: 14px;
    display: inline-flex;
    color: var(--accent);
    flex-shrink: 0;
  }
  .spec-icon :global(svg) {
    width: 100%;
    height: 100%;
  }
  .spec-value {
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }
  .spec-value.small {
    font-size: 11px;
    font-weight: 500;
    margin-top: 2px;
    color: var(--text2);
  }
  .spec-sub {
    font-size: 10px;
    color: var(--text3);
    margin-top: 3px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.02em;
    margin-bottom: 14px;
    color: var(--text);
  }
  .panel-title {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .spark {
    width: 13px;
    height: 13px;
    color: var(--accent);
    display: inline-flex;
  }
  .spark :global(svg) {
    width: 100%;
    height: 100%;
  }

  /* ── compact progress-bar rows, replacing the old circular speedometers ── */
  .bar-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .bar-row {
    padding: 2px 0;
  }
  .bar-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 5px;
  }
  .bar-name {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--text);
  }
  .bar-pct {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    flex-shrink: 0;
  }
  .bar-track {
    width: 100%;
    height: 7px;
    background: var(--bg3);
    border-radius: 4px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.4s ease;
  }
  .bar-skel-track {
    width: 100%;
    height: 7px;
    border-radius: 4px;
  }
  .bar-note {
    font-size: 9.5px;
    color: var(--text3);
    margin-top: 4px;
    line-height: 1.35;
  }

  .cta-panel {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    background: var(--bg2);
  }
  .cta-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }
  .cta-sub {
    font-size: 11px;
    color: var(--text3);
    margin-top: 2px;
  }

  .primary-btn {
    background: var(--accent);
    color: #000;
    border: none;
    border-radius: var(--r2);
    padding: 11px 18px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    cursor: pointer;
    transition: background 0.15s, transform 0.1s;
  }
  .primary-btn:hover:not(:disabled) {
    background: var(--accent2);
    transform: translateY(-1px);
  }
  .primary-btn:disabled {
    background: var(--bg4);
    color: var(--text3);
    cursor: not-allowed;
  }
  .run-btn {
    background: #111111;
    color: #fff;
    border: none;
    border-radius: var(--r2);
    padding: 8px 14px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.2s;
  }
  .run-btn:hover:not(:disabled) {
    background: #333;
  }
  .run-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .text-btn {
    background: none;
    border: none;
    color: var(--text3);
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    cursor: pointer;
    margin-top: 12px;
    transition: color 0.15s;
  }
  .text-btn:hover {
    color: var(--text);
  }

  /* ── app picker: a floating modal card over the main window (not part of
     it) — capped to the viewport height with its own scroll, so it never
     needs to resize the window and can never run off screen ── */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
    padding: 20px;
  }
  .modal {
    background: #fff;
    border: 1px solid var(--border);
    border-radius: var(--r4);
    padding: 22px;
    width: 500px;
    max-width: 100%;
    max-height: 100%;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.18);
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 700;
    margin-bottom: 14px;
    color: var(--text);
  }
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 13px;
    color: var(--text3);
    transition: color 0.15s;
  }
  .close-btn:hover {
    color: var(--text);
  }
  .modal-sub {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text3);
    margin-bottom: 10px;
  }
  .dept-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
  .dept-btn {
    display: flex;
    align-items: center;
    gap: 9px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--r2);
    padding: 11px 13px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.03em;
    color: var(--text2);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, color 0.15s;
  }
  .dept-btn:hover {
    border-color: var(--border2);
    color: var(--text);
  }
  .dept-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    display: flex;
  }
  .dept-icon :global(svg) {
    width: 16px;
    height: 16px;
  }

  .modal-topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .chip-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 6px 12px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text2);
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }
  .chip-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
  .app-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .app-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 7px 12px;
    font-family: var(--font-mono);
    font-size: 10px;
    background: transparent;
    color: var(--text2);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s, color 0.15s;
  }
  .app-pill:hover {
    border-color: var(--border2);
    color: var(--text);
  }
  .app-pill.active {
    border-color: var(--accent);
    background: rgba(255, 107, 0, 0.08);
    color: var(--accent);
  }
  .app-icon {
    width: 14px;
    height: 14px;
    border-radius: 3px;
  }
</style>