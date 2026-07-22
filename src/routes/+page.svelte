<script>
  import { onMount, onDestroy } from 'svelte'
  import { fade, fly, scale } from 'svelte/transition'
  import { invoke } from '@tauri-apps/api/core'
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

  // ── scan-line loading copy (purely cosmetic — cycles while we wait) ──
  const SPEC_PHRASES = [
    'Waking up the sensors…',
    'Reading CPU registers…',
    'Polling memory banks…',
    'Probing the GPU…',
    'Indexing storage volumes…',
    'Compiling machine profile…',
  ]
  let phraseIdx = 0
  let phraseTimer = null

  onMount(async () => {
    phraseTimer = setInterval(() => {
      phraseIdx = (phraseIdx + 1) % SPEC_PHRASES.length
    }, 950)

    try {
      specs = await invoke('get_system_specs')
    } catch (e) {
      specsError = String(e)
    } finally {
      specsLoading = false
      clearInterval(phraseTimer)
    }
    if (specs) runGeneral()
  })

  onDestroy(() => {
    if (phraseTimer) clearInterval(phraseTimer)
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

  // ── gauge geometry (ported from the site's speedometer) ──
  function polarPt(cx, cy, r, angleDeg) {
    const rad = (angleDeg * Math.PI) / 180
    return { x: cx + r * Math.cos(rad), y: cy - r * Math.sin(rad) }
  }
  function arcPath(cx, cy, r, a1, a2) {
    const p1 = polarPt(cx, cy, r, a1), p2 = polarPt(cx, cy, r, a2)
    return `M ${p1.x.toFixed(2)} ${p1.y.toFixed(2)} A ${r} ${r} 0 0 1 ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`
  }
  function needleTip(value) {
    const cx = 50, cy = 54, r = 40
    const angle = 180 - (value / 100) * 180
    return polarPt(cx, cy, r - 9, angle)
  }
  function gaugeColor(v) {
    return v >= 66 ? '#3DDC84' : v >= 33 ? '#8BE07A' : '#FF5C6C'
  }
  // Fallback description for the rare case the AI returns a score with no verdict text.
  function bandDescription(v) {
    if (v >= 71) return 'Smooth — handles this comfortably.'
    if (v >= 46) return 'Usable, but expect occasional slowdowns on heavier tasks.'
    if (v >= 21) return 'Workable, but expect jitters and longer render/export times.'
    return "Not really built for this — expect stutters and long waits."
  }
</script>

<main>
  <div class="brandbar">
    <img class="brand-logo" src={logo} alt="MakeDo logo" />
    <span class="brand-caption">built by <strong>makedo</strong></span>
  </div>

  <header>
    <h1>Benchy</h1>
    <p class="sub">Reads your real hardware, then estimates what it can actually handle.</p>
  </header>

  {#if specsLoading}
    <div class="panel scan-panel" transition:fade={{ duration: 200 }}>
      <div class="scan-ring">
        <svg viewBox="0 0 100 100" class="scan-ring-svg" aria-hidden="true">
          <circle cx="50" cy="50" r="42" class="scan-ring-track" />
          <circle cx="50" cy="50" r="42" class="scan-ring-sweep" />
        </svg>
        <span class="scan-ring-icon">{@html SPEC_ICONS.host}</span>
      </div>
      {#key phraseIdx}
        <div class="scan-label" in:fade={{ duration: 220 }}>{SPEC_PHRASES[phraseIdx]}</div>
      {/key}
    </div>
  {:else if specsError}
    <div class="panel error-panel">
      <span class="icon-lg">{@html UI_ICONS.alert}</span>
      <div>Couldn't read system specs.<div class="error-detail">{specsError}</div></div>
    </div>
  {:else if specs}
    <section class="panel specs-panel" in:fade={{ duration: 250 }}>
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
        {#if general.stage === 'loading'}<span class="tag">estimating…</span>{/if}
      </div>

      {#if general.stage === 'loading'}
        <div class="mini-scan" in:fade={{ duration: 150 }}>
          <div class="mini-ring">
            <svg viewBox="0 0 100 100" aria-hidden="true">
              <circle cx="50" cy="50" r="42" class="scan-ring-track" />
              <circle cx="50" cy="50" r="42" class="scan-ring-sweep" />
            </svg>
          </div>
          <div class="mini-scan-label">Modeling headroom for each category…</div>
          <div class="chip-row">
            {#each GENERAL_CATEGORIES as cat, i}
              <span class="chip" style="animation-delay: {i * 110}ms">{cat}</span>
            {/each}
          </div>
        </div>
      {:else if general.error}
        <div class="unavailable" in:fade={{ duration: 200 }}>
          <span class="icon-lg">{@html UI_ICONS.alert}</span>
          <div>
            <div class="unavailable-title">AI estimator unavailable</div>
            <div class="unavailable-sub">
              {general.errorDetail || "Couldn't reach the AI estimator, so no scores are shown."}
            </div>
          </div>
        </div>
      {:else}
        <div class="gauge-grid">
          {#each GENERAL_CATEGORIES as cat, i}
            {@const val = general.results[cat] ?? 0}
            {@const tip = needleTip(val)}
            <div class="gauge-card" in:fly={{ y: 8, duration: 220, delay: i * 40 }}>
              <svg viewBox="0 0 100 66" width="70" height="46">
                <path d={arcPath(50, 54, 40, 180, 120)} stroke="#FF5C6C" stroke-width="9" fill="none" stroke-linecap="round" />
                <path d={arcPath(50, 54, 40, 120, 60)} stroke="#8BE07A" stroke-width="9" fill="none" stroke-linecap="round" />
                <path d={arcPath(50, 54, 40, 60, 0)} stroke="#3DDC84" stroke-width="9" fill="none" stroke-linecap="round" />
                <line x1="50" y1="54" x2={tip.x.toFixed(2)} y2={tip.y.toFixed(2)} stroke="#EDEFF1" stroke-width="2.5" stroke-linecap="round" />
                <circle cx="50" cy="54" r="10" fill="#14171C" stroke="#333941" stroke-width="1" />
              </svg>
              <div class="gauge-name">{cat}</div>
              <div class="gauge-pct" style="color:{gaugeColor(val)}">{val}%</div>
              <div class="gauge-note">{general.notes[cat] || bandDescription(val)}</div>
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
      <div class="modal" on:click|stopPropagation transition:scale={{ start: 0.97, duration: 180 }}>
        <div class="modal-header">
          <span>App-specific benchmark</span>
          <button class="close-btn" on:click={closePicker}>✕</button>
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
            <button class="primary-btn small" disabled={!picker.apps.length} on:click={runAppBenchmark}>
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
          <div class="mini-scan" in:fade={{ duration: 150 }}>
            <div class="mini-ring">
              <svg viewBox="0 0 100 100" aria-hidden="true">
                <circle cx="50" cy="50" r="42" class="scan-ring-track" />
                <circle cx="50" cy="50" r="42" class="scan-ring-sweep" />
              </svg>
            </div>
            <div class="mini-scan-label">Checking this machine against your apps…</div>
            <div class="chip-row">
              {#each picker.apps as a, i}
                <span class="chip" style="animation-delay: {i * 110}ms">{a}</span>
              {/each}
            </div>
          </div>
        {:else if bench.error}
          <div class="unavailable" in:fade={{ duration: 200 }}>
            <span class="icon-lg">{@html UI_ICONS.alert}</span>
            <div>
              <div class="unavailable-title">AI estimator unavailable</div>
              <div class="unavailable-sub">
                {bench.errorDetail || "Couldn't reach the AI estimator, so no scores are shown."}
              </div>
            </div>
          </div>
          <button class="text-btn" on:click={backToDept}>← try different apps</button>
        {:else}
          <div class="gauge-grid">
            {#each picker.apps as a, i}
              {@const val = bench.results[a] ?? 0}
              {@const tip = needleTip(val)}
              <div class="gauge-card" in:fly={{ y: 8, duration: 220, delay: i * 40 }}>
                <svg viewBox="0 0 100 66" width="70" height="46">
                  <path d={arcPath(50, 54, 40, 180, 120)} stroke="#FF5C6C" stroke-width="9" fill="none" stroke-linecap="round" />
                  <path d={arcPath(50, 54, 40, 120, 60)} stroke="#8BE07A" stroke-width="9" fill="none" stroke-linecap="round" />
                  <path d={arcPath(50, 54, 40, 60, 0)} stroke="#3DDC84" stroke-width="9" fill="none" stroke-linecap="round" />
                  <line x1="50" y1="54" x2={tip.x.toFixed(2)} y2={tip.y.toFixed(2)} stroke="#EDEFF1" stroke-width="2.5" stroke-linecap="round" />
                  <circle cx="50" cy="54" r="10" fill="#14171C" stroke="#333941" stroke-width="1" />
                </svg>
                <div class="gauge-name">{a}</div>
                <div class="gauge-pct" style="color:{gaugeColor(val)}">{val}%</div>
                <div class="gauge-note">{bench.notes[a] || bandDescription(val)}</div>
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
  :global(body) {
    background:
      radial-gradient(1200px 600px at 15% -10%, rgba(255, 106, 43, 0.08), transparent 60%),
      #0B0D10;
    font-family: 'IBM Plex Mono', ui-monospace, monospace;
    color: #EDEFF1;
  }
  main {
    max-width: 760px;
    margin: 0 auto;
    padding: 28px 20px 60px;
  }

  /* ── brand bar ── */
  .brandbar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 22px;
  }
  .brand-logo {
    width: 22px;
    height: 22px;
    border-radius: 5px;
    display: block;
  }
  .brand-caption {
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #6B7178;
  }
  .brand-caption strong {
    color: #FF6A2B;
    font-weight: 600;
  }

  header h1 {
    font-size: 24px;
    margin: 0 0 4px;
    letter-spacing: 0.01em;
    color: #F5F6F8;
  }
  .sub {
    color: #868D97;
    font-size: 12px;
    margin: 0 0 20px;
  }

  .panel {
    background: #14171C;
    border: 1px solid #262B32;
    border-radius: 8px;
    padding: 18px 20px;
    margin-bottom: 16px;
  }

  /* ── full scan panel (initial specs read) ── */
  .scan-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 42px 10px;
  }
  .scan-ring {
    position: relative;
    width: 88px;
    height: 88px;
    margin-bottom: 16px;
  }
  .scan-ring-svg {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }
  .scan-ring-track {
    fill: none;
    stroke: #262B32;
    stroke-width: 4;
  }
  .scan-ring-sweep {
    fill: none;
    stroke: #FF6A2B;
    stroke-width: 4;
    stroke-linecap: round;
    stroke-dasharray: 44 220;
    filter: drop-shadow(0 0 5px rgba(255, 106, 43, 0.55));
    animation: spin 1.3s linear infinite;
    transform-origin: 50px 50px;
  }
  .scan-ring-icon {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #5B6169;
  }
  .scan-ring-icon :global(svg) {
    width: 26px;
    height: 26px;
    animation: pulseSoft 1.8s ease-in-out infinite;
  }
  .scan-label {
    font-size: 11.5px;
    color: #9BA1AA;
    letter-spacing: 0.02em;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes pulseSoft { 0%, 100% { opacity: 0.45; } 50% { opacity: 0.95; } }

  /* ── mini scan (general/bench "thinking") ── */
  .mini-scan {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 26px 10px 8px;
    text-align: center;
  }
  .mini-ring {
    width: 46px;
    height: 46px;
    margin-bottom: 10px;
  }
  .mini-ring svg { width: 100%; height: 100%; transform: rotate(-90deg); }
  .mini-scan-label {
    font-size: 11px;
    color: #9BA1AA;
    margin-bottom: 14px;
  }
  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    justify-content: center;
  }
  .chip {
    font-size: 9.5px;
    letter-spacing: 0.03em;
    color: #9BA1AA;
    background: #1A1E24;
    border: 1px solid #262B32;
    border-radius: 20px;
    padding: 4px 10px;
    animation: chipPulse 1.6s ease-in-out infinite;
  }
  @keyframes chipPulse { 0%, 100% { opacity: 0.35; } 50% { opacity: 1; } }

  .error-panel {
    text-align: center;
    font-size: 12px;
    padding: 30px 10px;
    color: #FF8C93;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .error-detail {
    color: #868D97;
    font-size: 10.5px;
    margin-top: 4px;
  }

  /* ── unavailable state (replaces made-up numbers) ── */
  .unavailable {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    background: #1A1512;
    border: 1px solid #3A2A1E;
    border-radius: 6px;
    padding: 14px 16px;
  }
  .unavailable-title {
    font-size: 12px;
    font-weight: 600;
    color: #FFB27A;
    margin-bottom: 3px;
  }
  .unavailable-sub {
    font-size: 10.5px;
    color: #B49180;
    line-height: 1.5;
  }
  .icon-lg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    color: #FF8C4B;
  }
  .icon-lg :global(svg) { width: 100%; height: 100%; }

  .specs-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    color: #F5F6F8;
    margin-bottom: 14px;
  }
  .spec-icon.inline { color: #FF6A2B; }
  .os-tag {
    font-weight: 400;
    color: #6B7178;
    font-size: 11px;
    margin-left: 4px;
  }
  .specs-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }
  .spec-card {
    background: #1A1E24;
    border: 1px solid #262B32;
    border-radius: 6px;
    padding: 12px 14px;
    transition: border-color 0.15s;
  }
  .spec-card:hover { border-color: #3A4048; }
  .spec-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #868D97;
    margin-bottom: 6px;
  }
  .spec-icon {
    width: 14px;
    height: 14px;
    display: inline-flex;
    color: #FF6A2B;
    flex-shrink: 0;
  }
  .spec-icon :global(svg) { width: 100%; height: 100%; }
  .spec-value {
    font-size: 12px;
    font-weight: 600;
    color: #EDEFF1;
  }
  .spec-value.small { font-size: 11px; font-weight: 500; margin-top: 2px; color: #C6CAD1; }
  .spec-sub { font-size: 10px; color: #6B7178; margin-top: 3px; }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.03em;
    margin-bottom: 14px;
    color: #F5F6F8;
  }
  .panel-title { display: flex; align-items: center; gap: 7px; }
  .spark { width: 13px; height: 13px; color: #FF6A2B; display: inline-flex; }
  .spark :global(svg) { width: 100%; height: 100%; }
  .tag {
    font-weight: 400;
    color: #FF6A2B;
    font-size: 10px;
  }

  .gauge-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
  }
  .gauge-card {
    background: #1A1E24;
    border: 1px solid #262B32;
    border-radius: 6px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    transition: border-color 0.15s;
  }
  .gauge-card:hover { border-color: #3A4048; }
  .gauge-name { font-size: 10px; font-weight: 600; margin-top: 5px; color: #EDEFF1; }
  .gauge-pct { font-size: 14px; font-weight: 700; margin-top: 2px; }
  .gauge-note { font-size: 9px; color: #868D97; margin-top: 4px; line-height: 1.35; }

  .cta-panel {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    background: linear-gradient(135deg, #171A1F, #14171C);
  }
  .cta-title { font-size: 13px; font-weight: 600; color: #F5F6F8; }
  .cta-sub { font-size: 11px; color: #868D97; margin-top: 2px; }

  .primary-btn {
    background: #FF6A2B;
    color: #14100C;
    border: none;
    border-radius: 5px;
    padding: 11px 18px;
    font-family: inherit;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.03em;
    cursor: pointer;
    box-shadow: 0 0 0 rgba(255, 106, 43, 0);
    transition: box-shadow 0.15s, transform 0.1s;
  }
  .primary-btn:hover:not(:disabled) {
    box-shadow: 0 0 18px rgba(255, 106, 43, 0.35);
    transform: translateY(-1px);
  }
  .primary-btn.small { padding: 8px 14px; }
  .primary-btn:disabled { background: #2A2E34; color: #6B7178; cursor: not-allowed; }
  .text-btn {
    background: none;
    border: none;
    color: #868D97;
    font-family: inherit;
    font-size: 10.5px;
    cursor: pointer;
    margin-top: 12px;
  }
  .text-btn:hover { color: #EDEFF1; }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(4, 5, 6, 0.65);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
  }
  .modal {
    background: #14171C;
    border: 1px solid #262B32;
    border-radius: 10px;
    padding: 22px;
    width: 520px;
    max-width: 92vw;
    max-height: 85vh;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    font-weight: 700;
    margin-bottom: 14px;
    color: #F5F6F8;
  }
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 13px;
    color: #868D97;
  }
  .close-btn:hover { color: #EDEFF1; }
  .modal-sub {
    font-size: 10.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #6B7178;
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
    background: #1A1E24;
    border: 1px solid #262B32;
    border-radius: 6px;
    padding: 11px 13px;
    font-family: inherit;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.02em;
    color: #C6CAD1;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, color 0.15s;
  }
  .dept-btn:hover { border-color: #FF6A2B; color: #FF6A2B; }
  .dept-icon { width: 16px; height: 16px; flex-shrink: 0; display: flex; }
  .dept-icon :global(svg) { width: 16px; height: 16px; }

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
    background: #1A1E24;
    border: 1px solid #262B32;
    border-radius: 20px;
    padding: 6px 12px;
    font-family: inherit;
    font-size: 10px;
    color: #C6CAD1;
    cursor: pointer;
  }
  .chip-btn:hover { border-color: #3A4048; }
  .app-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .app-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    border: 1px solid #262B32;
    border-radius: 20px;
    padding: 7px 12px;
    font-family: inherit;
    font-size: 10px;
    background: #1A1E24;
    color: #C6CAD1;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s, color 0.15s;
  }
  .app-pill:hover { border-color: #3A4048; }
  .app-pill.active {
    border-color: #FF6A2B;
    background: rgba(255, 106, 43, 0.1);
    color: #FF9E5C;
  }
  .app-icon { width: 14px; height: 14px; border-radius: 3px; }
</style>