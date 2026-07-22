<script>
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { DEPARTMENTS, DEPT_ICONS, APP_ICONS, GENERAL_CATEGORIES } from '$lib/perfData.js'

  // ── specs ──
  let specs = null
  let specsError = ''
  let specsLoading = true

  // ── general readiness (auto-run once specs land) ──
  let general = { stage: 'idle', results: {}, notes: {}, error: false } // idle|loading|done

  // ── app-specific picker ──
  let picker = { open: false, step: 'dept', dept: null, apps: [] } // step: dept|apps
  let bench = { stage: 'idle', results: {}, notes: {}, error: false } // idle|loading|done

  onMount(async () => {
    try {
      specs = await invoke('get_system_specs')
    } catch (e) {
      specsError = String(e)
    } finally {
      specsLoading = false
    }
    if (specs) runGeneral()
  })

  async function runGeneral() {
    general.stage = 'loading'
    general = general
    try {
      const resp = await invoke('benchmark_apps', { specs, dept: null, apps: GENERAL_CATEGORIES })
      const results = {}, notes = {}
      GENERAL_CATEGORIES.forEach(c => {
        const entry = resp.results[c]
        results[c] = entry ? entry.score : 0
        notes[c] = entry ? entry.verdict : ''
      })
      general = { stage: 'done', results, notes, error: resp.estimator === 'fallback' }
    } catch (e) {
      console.error('general benchmark failed', e)
      general = { stage: 'done', results: {}, notes: {}, error: true }
    }
  }

  function openPicker() {
    picker = { open: true, step: 'dept', dept: null, apps: [] }
    bench = { stage: 'idle', results: {}, notes: {}, error: false }
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
    bench.stage = 'loading'
    bench = bench
    try {
      const resp = await invoke('benchmark_apps', {
        specs,
        dept: picker.dept.label,
        apps: picker.apps,
      })
      const results = {}, notes = {}
      picker.apps.forEach(a => {
        const entry = resp.results[a]
        results[a] = entry ? entry.score : 0
        notes[a] = entry ? entry.verdict : ''
      })
      bench = { stage: 'done', results, notes, error: resp.estimator === 'fallback' }
    } catch (e) {
      console.error('app benchmark failed', e)
      bench = { stage: 'done', results: {}, notes: {}, error: true }
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
    return v >= 66 ? '#16A34A' : v >= 33 ? '#65A30D' : '#EF4444'
  }
  // Fallback description when the AI didn't return a verdict (offline / key missing).
  function bandDescription(v) {
    if (v >= 71) return 'Smooth — handles this comfortably.'
    if (v >= 46) return 'Usable, but expect occasional slowdowns on heavier tasks.'
    if (v >= 21) return 'Workable, but expect jitters and longer render/export times.'
    return "Not really built for this — expect stutters and long waits."
  }
</script>

<main>
  <header>
    <h1>Benchy</h1>
    <p class="sub">Reads your real hardware, then estimates what it can actually handle.</p>
  </header>

  {#if specsLoading}
    <div class="panel loading-panel">Reading your machine's specs…</div>
  {:else if specsError}
    <div class="panel error-panel">Couldn't read system specs: {specsError}</div>
  {:else if specs}
    <section class="panel specs-panel">
      <div class="specs-title">{specs.hostname} <span class="os-tag">{specs.os} {specs.os_version}</span></div>
      <div class="specs-grid">
        <div class="spec-card">
          <div class="spec-label">CPU</div>
          <div class="spec-value">{specs.cpu_brand}</div>
          <div class="spec-sub">{specs.cpu_cores} cores / {specs.cpu_threads} threads</div>
        </div>
        <div class="spec-card">
          <div class="spec-label">RAM</div>
          <div class="spec-value">{specs.ram_total_gb} GB</div>
          <div class="spec-sub">{specs.ram_available_gb} GB available now</div>
        </div>
        <div class="spec-card">
          <div class="spec-label">GPU</div>
          <div class="spec-value">{specs.gpu_name}</div>
        </div>
        <div class="spec-card">
          <div class="spec-label">Storage</div>
          {#each specs.disks.slice(0, 2) as d}
            <div class="spec-value small">{d.name || 'Disk'}: {d.total_gb} GB ({d.kind})</div>
          {/each}
        </div>
      </div>
    </section>

    <section class="panel general-panel">
      <div class="panel-header">
        <span>General readiness</span>
        {#if general.stage === 'loading'}<span class="tag">estimating…</span>{/if}
      </div>

      {#if general.error && general.stage === 'done'}
        <div class="note">Couldn't reach the AI estimator — showing a rough estimate from specs alone.</div>
      {/if}

      <div class="gauge-grid">
        {#each GENERAL_CATEGORIES as cat}
          {#if general.stage === 'loading'}
            <div class="gauge-card skeleton"></div>
          {:else}
            {@const val = general.results[cat] ?? 0}
            {@const tip = needleTip(val)}
            <div class="gauge-card">
              <svg viewBox="0 0 100 66" width="70" height="46">
                <path d={arcPath(50, 54, 40, 180, 120)} stroke="#EF4444" stroke-width="9" fill="none" stroke-linecap="round" />
                <path d={arcPath(50, 54, 40, 120, 60)} stroke="#86EFAC" stroke-width="9" fill="none" stroke-linecap="round" />
                <path d={arcPath(50, 54, 40, 60, 0)} stroke="#16A34A" stroke-width="9" fill="none" stroke-linecap="round" />
                <line x1="50" y1="54" x2={tip.x.toFixed(2)} y2={tip.y.toFixed(2)} stroke="#111" stroke-width="2.5" stroke-linecap="round" />
                <circle cx="50" cy="54" r="10" fill="#fff" stroke="#ddd" stroke-width="1" />
              </svg>
              <div class="gauge-name">{cat}</div>
              <div class="gauge-pct" style="color:{gaugeColor(val)}">{val}%</div>
              <div class="gauge-note">{general.notes[cat] || bandDescription(val)}</div>
            </div>
          {/if}
        {/each}
      </div>
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
    <div class="modal-backdrop" on:click={closePicker}>
      <div class="modal" on:click|stopPropagation>
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
            <button class="chip" on:click={backToDept}>
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
          <div class="loading-panel">Checking this machine against your apps…</div>
        {:else}
          {#if bench.error}
            <div class="note">Couldn't reach the AI estimator — showing a rough estimate instead.</div>
          {/if}
          <div class="gauge-grid">
            {#each picker.apps as a}
              {@const val = bench.results[a] ?? 0}
              {@const tip = needleTip(val)}
              <div class="gauge-card">
                <svg viewBox="0 0 100 66" width="70" height="46">
                  <path d={arcPath(50, 54, 40, 180, 120)} stroke="#EF4444" stroke-width="9" fill="none" stroke-linecap="round" />
                  <path d={arcPath(50, 54, 40, 120, 60)} stroke="#86EFAC" stroke-width="9" fill="none" stroke-linecap="round" />
                  <path d={arcPath(50, 54, 40, 60, 0)} stroke="#16A34A" stroke-width="9" fill="none" stroke-linecap="round" />
                  <line x1="50" y1="54" x2={tip.x.toFixed(2)} y2={tip.y.toFixed(2)} stroke="#111" stroke-width="2.5" stroke-linecap="round" />
                  <circle cx="50" cy="54" r="10" fill="#fff" stroke="#ddd" stroke-width="1" />
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
    background: #fafafa;
    font-family: 'IBM Plex Mono', ui-monospace, monospace;
    color: #1a1a1a;
  }
  main {
    max-width: 760px;
    margin: 0 auto;
    padding: 32px 20px 60px;
  }
  header h1 {
    font-size: 22px;
    margin: 0 0 4px;
    letter-spacing: 0.02em;
  }
  .sub {
    color: #777;
    font-size: 12px;
    margin: 0 0 20px;
  }
  .panel {
    background: #fff;
    border: 1px solid #e4e4e4;
    border-radius: 4px;
    padding: 16px 18px;
    margin-bottom: 16px;
  }
  .loading-panel, .error-panel {
    text-align: center;
    color: #888;
    font-size: 12px;
    padding: 30px 10px;
  }
  .error-panel { color: #b91c1c; }

  .specs-title {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 12px;
  }
  .os-tag {
    font-weight: 400;
    color: #999;
    font-size: 11px;
    margin-left: 8px;
  }
  .specs-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }
  .spec-card {
    border: 1px solid #eee;
    border-radius: 3px;
    padding: 10px 12px;
  }
  .spec-label {
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #999;
    margin-bottom: 4px;
  }
  .spec-value {
    font-size: 12px;
    font-weight: 600;
  }
  .spec-value.small { font-size: 11px; font-weight: 500; margin-top: 2px; }
  .spec-sub { font-size: 10px; color: #999; margin-top: 2px; }

  .panel-header {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.04em;
    margin-bottom: 12px;
  }
  .tag {
    font-weight: 400;
    color: #ff6b00;
    font-size: 10px;
  }
  .note {
    font-size: 10.5px;
    color: #b45309;
    background: #fffbeb;
    border: 1px solid #fde68a;
    border-radius: 3px;
    padding: 6px 10px;
    margin-bottom: 10px;
  }

  .gauge-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
  }
  .gauge-card {
    border: 1px solid #eee;
    border-radius: 3px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  .gauge-card.skeleton {
    min-height: 110px;
    background: linear-gradient(90deg, #f2f2f2 25%, #e9e9e9 50%, #f2f2f2 75%);
    background-size: 200% 100%;
    animation: shimmer 1.3s infinite;
  }
  @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
  .gauge-name { font-size: 10px; font-weight: 600; margin-top: 4px; }
  .gauge-pct { font-size: 13px; font-weight: 700; }
  .gauge-note { font-size: 9px; color: #999; margin-top: 3px; line-height: 1.3; }

  .cta-panel {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .cta-title { font-size: 13px; font-weight: 600; }
  .cta-sub { font-size: 11px; color: #888; margin-top: 2px; }

  .primary-btn {
    background: #ff6b00;
    color: #fff;
    border: none;
    border-radius: 3px;
    padding: 10px 16px;
    font-family: inherit;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.03em;
    cursor: pointer;
  }
  .primary-btn.small { padding: 7px 12px; }
  .primary-btn:disabled { background: #ddd; cursor: not-allowed; }
  .text-btn {
    background: none;
    border: none;
    color: #888;
    font-family: inherit;
    font-size: 10.5px;
    cursor: pointer;
    margin-top: 10px;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
  }
  .modal {
    background: #fff;
    border-radius: 5px;
    padding: 20px;
    width: 520px;
    max-width: 92vw;
    max-height: 85vh;
    overflow-y: auto;
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    font-weight: 700;
    margin-bottom: 14px;
  }
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 13px;
    color: #999;
  }
  .modal-sub {
    font-size: 10.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #999;
    margin-bottom: 8px;
  }
  .dept-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
  .dept-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: 1px solid #ddd;
    border-radius: 3px;
    padding: 10px 12px;
    font-family: inherit;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.03em;
    color: #444;
    cursor: pointer;
    text-align: left;
  }
  .dept-btn:hover { border-color: #ff6b00; color: #ff6b00; }
  .dept-icon { width: 16px; height: 16px; flex-shrink: 0; display: flex; }
  .dept-icon :global(svg) { width: 16px; height: 16px; }

  .modal-topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .chip {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid #ddd;
    border-radius: 3px;
    padding: 6px 10px;
    font-family: inherit;
    font-size: 10px;
    color: #666;
    cursor: pointer;
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
    border: 1px solid #ddd;
    border-radius: 3px;
    padding: 6px 10px;
    font-family: inherit;
    font-size: 10px;
    background: #fff;
    cursor: pointer;
  }
  .app-pill.active {
    border-color: #ff6b00;
    background: rgba(255, 107, 0, 0.08);
    color: #ff6b00;
  }
  .app-icon { width: 14px; height: 14px; border-radius: 2px; }
</style>
