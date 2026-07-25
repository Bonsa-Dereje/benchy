<!-- src/routes/mainDash/+page.svelte -->
<script>
  import { onMount } from 'svelte'
  import { get } from 'svelte/store'
  import { fade, fly } from 'svelte/transition'
  import { goto } from '$app/navigation'
  import { invoke } from '@tauri-apps/api/core'
  import { DEPARTMENTS, DEPT_ICONS, APP_ICONS, SPEC_ICONS, UI_ICONS, GENERAL_CATEGORIES } from '$lib/perfData.js'
  import { specsStore, specsErrorStore, generalStore } from '$lib/loadStore.js'
  import logo from '../../assets/logo.png'

  // ── specs + general readiness ──
  // Both are fetched by src/routes/+page.svelte (the loading screen) *before* it
  // routes here, so by the time this page mounts they're already sitting
  // in the shared store — this page just reads them back out.
  let specs = null
  let specsError = ''
  let general = { stage: 'idle', results: {}, notes: {}, error: false, errorDetail: '' } // idle|loading|done

  // ── app-specific picker ──
  let picker = { open: false, step: 'dept', dept: null, apps: [] } // step: dept|apps
  let bench = { stage: 'idle', results: {}, notes: {}, error: false, errorDetail: '' } // idle|loading|done

  // ── battery health (runs `powercfg /batteryreport`, Windows-only) ──
  let battery = { stage: 'idle', data: null, error: false, errorDetail: '' } // idle|loading|done
  const BATTERY_ICON = '<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="7" width="18" height="10" rx="2" stroke="currentColor" stroke-width="1.8"/><rect x="21" y="10" width="2" height="4" rx="0.6" fill="currentColor"/><rect x="4.5" y="9.5" width="9" height="5" rx="0.8" fill="currentColor"/></svg>'

  async function runBatteryReport() {
    battery = { stage: 'loading', data: null, error: false, errorDetail: '' }
    try {
      const resp = await invoke('get_battery_report')
      battery = { stage: 'done', data: resp, error: false, errorDetail: '' }
    } catch (e) {
      console.error('battery report failed', e)
      battery = { stage: 'done', data: null, error: true, errorDetail: String(e) }
    }
  }

  // ── hardware upgrade advisor ──
  let upgradeModalOpen = false
  let upgradeResultsModalOpen = false
  let selectedUpgradeCat = null
  let selectedUpgrades = []
  let upgradeState = { stage: 'idle', data: null, error: false, errorDetail: '' } // idle|loading|done

  function openUpgradeModal() {
    upgradeModalOpen = true
    upgradeResultsModalOpen = false
    selectedUpgradeCat = null
    selectedUpgrades = []
    upgradeState = { stage: 'idle', data: null, error: false, errorDetail: '' }
  }

  function closeAllUpgradeModals() {
    upgradeModalOpen = false
    upgradeResultsModalOpen = false
  }

  function backToCategorySelect() {
    upgradeResultsModalOpen = false
    upgradeModalOpen = true
  }

  async function selectUpgradeCategory(cat) {
    selectedUpgradeCat = cat
    selectedUpgrades = []
    upgradeModalOpen = false
    upgradeResultsModalOpen = true
    upgradeState = { stage: 'loading', data: null, error: false, errorDetail: '' }

    try {
      const resp = await invoke('get_upgrade_advice', { specs, category: cat })
      upgradeState = { stage: 'done', data: resp, error: false, errorDetail: '' }
    } catch (e) {
      console.error('get_upgrade_advice failed', e)
      upgradeState = { stage: 'done', data: null, error: true, errorDetail: String(e) }
    }
  }

  function toggleUpgradeSelection(title) {
    if (selectedUpgrades.includes(title)) {
      selectedUpgrades = selectedUpgrades.filter(t => t !== title)
    } else {
      selectedUpgrades = [...selectedUpgrades, title]
    }
  }

  function proceedWithSelectedUpgrades() {
    if (!selectedUpgrades.length) return
    const itemsParam = selectedUpgrades.join('|')
    const cat = selectedUpgradeCat || 'PC performance'
    goto(`/redirect?item=${encodeURIComponent(itemsParam)}&category=${encodeURIComponent(cat)}`)
  }

  // ─────────────────────────────────────────────────────────────
  // Window size is hardcoded to match src-tauri/tauri.conf.json
  // (525×647, resizable: false) instead of being measured/scaled at
  // runtime — the previous "measure content, then shrink+resize the
  // window to fit" approach depended on the Tauri monitor/resize APIs
  // behaving the same across machines, which they didn't. Now the
  // layout itself is sized to fit inside that fixed window on every
  // machine, and the CSS just leaves it a little internal scroll room
  // in case a particular OS/font stack renders a touch taller.
  // ─────────────────────────────────────────────────────────────

  // ─────────────────────────────────────────────────────────────
  // Custom scrollbar + edge fade for the fixed 525×647 window.
  // `main` is a fixed-size viewport; `.scroll-area` inside it does the
  // actual scrolling with the native scrollbar hidden. A single pill
  // thumb is drawn on top (purely a position indicator, not draggable)
  // and only fades in while the user is actively scrolling or moving
  // the mouse over the window, then fades back out. Two soft gradients
  // at the top/bottom hint that content continues past the edge, and
  // hide themselves once you've scrolled all the way to that end.
  // ─────────────────────────────────────────────────────────────
  let scrollAreaEl
  let contentEl
  let contentObserver
  let scrollbarTimer

  let atTop = true
  let atBottom = true
  let thumbVisible = false
  let thumbTop = 0
  let thumbHeight = 0
  let scrollbarActive = false

  const THUMB_MIN = 28
  const TRACK_INSET = 6

  function updateScrollMetrics() {
    if (!scrollAreaEl) return
    const { scrollTop, scrollHeight, clientHeight } = scrollAreaEl
    const scrollable = scrollHeight > clientHeight + 1
    thumbVisible = scrollable
    atTop = !scrollable || scrollTop <= 1
    atBottom = !scrollable || scrollTop + clientHeight >= scrollHeight - 1

    if (scrollable) {
      const trackH = clientHeight - TRACK_INSET * 2
      thumbHeight = Math.max(THUMB_MIN, (clientHeight / scrollHeight) * trackH)
      const maxScroll = scrollHeight - clientHeight
      const travel = trackH - thumbHeight
      thumbTop = TRACK_INSET + (maxScroll > 0 ? (scrollTop / maxScroll) * travel : 0)
    }
  }

  function pulseScrollbar() {
    scrollbarActive = true
    clearTimeout(scrollbarTimer)
    scrollbarTimer = setTimeout(() => { scrollbarActive = false }, 900)
  }

  function onScroll() {
    updateScrollMetrics()
    pulseScrollbar()
  }

  function onMouseMove() {
    if (thumbVisible) pulseScrollbar()
  }

  function scrollDown() {
    if (!scrollAreaEl) return
    scrollAreaEl.scrollBy({ top: Math.round(scrollAreaEl.clientHeight * 0.8), behavior: 'smooth' })
  }

  onMount(() => {
    // Landed here without going through the loading screen (e.g. a hard
    // refresh, or opening '/' directly) — there's nothing to show yet, so
    // send back to the loading screen to fetch everything first.
    if (!get(specsStore) && !get(specsErrorStore)) {
      goto('/')
      return
    }

    specs = get(specsStore)
    specsError = get(specsErrorStore)
    general = get(generalStore)

    // Recompute whenever the actual content height changes (specs
    // arriving, readiness bars appearing, the app picker opening) —
    // not on window resize, since the window never resizes now.
    contentObserver = new ResizeObserver(updateScrollMetrics)
    contentObserver.observe(contentEl)
    updateScrollMetrics()

    return () => {
      contentObserver?.disconnect()
      clearTimeout(scrollbarTimer)
    }
  })

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

<main on:mousemove={onMouseMove}>
  <div class="scroll-area" bind:this={scrollAreaEl} on:scroll={onScroll}>
    <div class="scroll-content" bind:this={contentEl}>
      <div class="brandbar">
        <img class="brand-logo" src={logo} alt="MakeDo logo" />
        <span class="brand-caption">built by <strong>makedo</strong></span>
      </div>

      <header>
        <h1>Benchy</h1>
        <p class="sub">Reads your real hardware, then estimates what it can actually handle.</p>
      </header>

  {#if specsError}
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

    {#if specs.os && specs.os.toLowerCase().includes('windows')}
      <section class="panel battery-panel" in:fade={{ duration: 220 }}>
        <div class="panel-header">
          <span class="panel-title"><span class="spark">{@html BATTERY_ICON}</span>Battery health</span>
          {#if battery.stage === 'done' && !battery.error}
            <button class="text-btn battery-refresh" on:click={runBatteryReport}>Refresh</button>
          {/if}
        </div>

        {#if battery.stage === 'idle'}
          <div class="battery-idle">
            <div class="cta-sub">
              Runs Windows' built-in battery report and compares full-charge capacity against
              design capacity to see how much the battery has worn down.
            </div>
            <button class="run-btn" on:click={runBatteryReport}>Check battery health →</button>
          </div>
        {:else if battery.stage === 'loading'}
          <div class="loading-panel" in:fade={{ duration: 150 }}>
            <div class="ai-pulse"><span class="spark-big">{@html BATTERY_ICON}</span></div>
            <div class="loading-title">Generating battery report…</div>
            <div class="loading-sub">Running powercfg /batteryreport</div>
          </div>
        {:else if battery.error}
          <div class="unavailable" in:fade={{ duration: 200 }}>
            <span class="icon-lg">{@html UI_ICONS.alert}</span>
            <div>
              <div class="unavailable-title">Couldn't read battery report</div>
              <div class="unavailable-sub">
                {battery.errorDetail || 'powercfg failed, or no report was generated.'}
              </div>
            </div>
          </div>
          <button class="text-btn" on:click={runBatteryReport}>← try again</button>
        {:else if battery.data && !battery.data.has_battery}
          <div class="battery-none">No battery detected — this looks like a desktop.</div>
        {:else if battery.data}
          {@const d = battery.data}
          {@const health = d.health_pct ?? 0}
          {@const designWh = d.design_capacity_mwh != null ? d.design_capacity_mwh / 1000 : null}
          {@const fullWh = d.full_charge_capacity_mwh != null ? d.full_charge_capacity_mwh / 1000 : null}
          <div class="battery-caps" in:fade={{ duration: 200 }}>
            <div class="battery-cap">
              <div class="battery-cap-label">Design capacity</div>
              <div class="battery-cap-value">{designWh?.toFixed(1)} Wh</div>
            </div>
            <div class="battery-cap">
              <div class="battery-cap-label">Full charge capacity</div>
              <div class="battery-cap-value">{fullWh?.toFixed(1)} Wh</div>
            </div>
            {#if d.cycle_count != null}
              <div class="battery-cap">
                <div class="battery-cap-label">Cycle count</div>
                <div class="battery-cap-value">{d.cycle_count}</div>
              </div>
            {/if}
          </div>
          <div class="bar-row">
            <div class="bar-top">
              <span class="bar-name">Battery health</span>
              <span class="bar-pct" style="color:{gaugeColor(health)}">{health.toFixed(0)}%</span>
            </div>
            <div class="bar-track">
              <div class="bar-fill" style="width:{health}%; background:{gaugeColor(health)}"></div>
            </div>
            <div class="bar-note">
              Degraded {d.degradation_pct?.toFixed(1)}% from design capacity
              ({designWh?.toFixed(1)} Wh → {fullWh?.toFixed(1)} Wh).
            </div>
          </div>
          <div class="battery-upgrade-action">
            <button class="upgrade-battery-btn" on:click={() => selectUpgradeCategory('Battery Health')}>
              Upgrade battery →
            </button>
          </div>
        {/if}
      </section>
    {/if}

    <section class="panel general-panel">
      <div class="panel-header">
        <span class="panel-title"><span class="spark">{@html UI_ICONS.spark}</span>PC performance</span>
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
        <div class="panel-bottom-action">
          <button class="upgrade-pc-btn" on:click={openUpgradeModal}>
            Upgrade your PC →
          </button>
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
    </div>
  </div>

  <div class="edge-fade edge-fade-top" class:hidden={atTop}></div>
  <div class="edge-fade edge-fade-bottom" class:hidden={atBottom}></div>
  <div
    class="scroll-thumb"
    class:visible={thumbVisible && scrollbarActive}
    style="top:{thumbTop}px; height:{thumbHeight}px;"
  ></div>

  <button
    class="scroll-down-btn"
    class:hidden={!thumbVisible || atBottom}
    on:click={scrollDown}
    aria-label="Scroll down"
    tabindex={!thumbVisible || atBottom ? -1 : 0}
  >
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M6 10L12 16L18 10" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>

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

  <!-- Modal 1: Select Category to Improve -->
  {#if upgradeModalOpen}
    <div class="modal-backdrop" on:click={closeAllUpgradeModals} transition:fade={{ duration: 150 }}>
      <div class="modal upgrade-modal" on:click|stopPropagation transition:fly={{ y: 12, duration: 200 }}>
        <div class="modal-header">
          <span>Hardware Upgrade Advisor</span>
          <button class="close-btn" on:click={closeAllUpgradeModals} aria-label="Close">✕</button>
        </div>

        <div class="pulse-box">
          <div class="pulse-inner">
            <div>
              <div class="pulse-title">Which one do you wanna improve?</div>
              <div class="pulse-sub">Select any performance standard below to analyze hardware upgrade options</div>
            </div>
          </div>
        </div>

        <div class="bar-list modal-bar-list">
          {#each GENERAL_CATEGORIES as cat, i}
            {@const val = general.results[cat] ?? 0}
            <button
              class="bar-row bar-row-interactive"
              on:click={() => selectUpgradeCategory(cat)}
              in:fly={{ y: 6, duration: 220, delay: i * 30 }}
            >
              <div class="bar-top">
                <span class="bar-name">{cat}</span>
                <div class="bar-top-right">
                  <span class="bar-pct" style="color:{gaugeColor(val)}">{val}%</span>
                  <span class="select-arrow">→</span>
                </div>
              </div>
              <div class="bar-track">
                <div class="bar-fill" style="width:{val}%; background:{gaugeColor(val)}"></div>
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  <!-- Modal 2: Upgrade Analysis & Multi-Select Options -->
  {#if upgradeResultsModalOpen}
    <div class="modal-backdrop" on:click={closeAllUpgradeModals} transition:fade={{ duration: 150 }}>
      <div class="modal upgrade-modal" on:click|stopPropagation transition:fly={{ y: 12, duration: 200 }}>
        <div class="modal-header">
          <button class="back-link-btn" on:click={backToCategorySelect}>← Select standard</button>
          <button class="close-btn" on:click={closeAllUpgradeModals} aria-label="Close">✕</button>
        </div>

        <div class="modal-category-header">
          <div class="modal-sub">Performance Target</div>
          <div class="modal-cat-title">{selectedUpgradeCat}</div>
        </div>

        {#if upgradeState.stage === 'loading'}
          <div class="upgrade-loading-modal" in:fade={{ duration: 150 }}>
            <div class="analyzing-spinner">
              <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="2" x2="12" y2="6"/>
                <line x1="12" y1="18" x2="12" y2="22"/>
                <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"/>
                <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"/>
                <line x1="2" y1="12" x2="6" y2="12"/>
                <line x1="18" y1="12" x2="22" y2="12"/>
                <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"/>
                <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"/>
              </svg>
            </div>
            <div class="loading-title">Analyzing hardware bottlenecks…</div>
            <div class="loading-sub">Evaluating specs for {selectedUpgradeCat}</div>
          </div>
        {:else if upgradeState.stage === 'done' && upgradeState.data}
          <div class="upgrade-summary-box">
            <span class="lightbulb-icon">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 18h6M10 22h4M15 14.5c.831-.734 1.5-1.93 1.5-3.5A4.5 4.5 0 0 0 7.5 11c0 1.57.669 2.766 1.5 3.5.5.44.86 1.01.86 1.7V17h4.28v-.8c0-.69.36-1.26.86-1.7z"/>
              </svg>
            </span>
            <span>{upgradeState.data.summary}</span>
          </div>

          <div class="upgrade-options-title">Recommended Upgrades (Select all that apply):</div>
          <div class="upgrade-cards-list">
            {#each upgradeState.data.upgrades as upg}
              {@const isSelected = selectedUpgrades.includes(upg.title)}
              <button
                class="upgrade-card-btn {isSelected ? 'selected' : ''}"
                on:click={() => toggleUpgradeSelection(upg.title)}
              >
                <div class="card-header-row">
                  <div class="header-left">
                    <div class="custom-checkbox {isSelected ? 'checked' : ''}">
                      {#if isSelected}✓{/if}
                    </div>
                    <span class="comp-badge {upg.component.toLowerCase()}">{upg.component}</span>
                  </div>
                  <span class="boost-badge">{upg.estimated_boost}</span>
                </div>
                <div class="card-item-title">{upg.title}</div>
                <div class="card-item-reason">{upg.reason}</div>
              </button>
            {/each}
          </div>

          <div class="proceed-footer">
            <button
              class="proceed-btn"
              disabled={!selectedUpgrades.length}
              on:click={proceedWithSelectedUpgrades}
            >
              Proceed {selectedUpgrades.length ? `(${selectedUpgrades.length} selected)` : ''} →
            </button>
          </div>
        {:else if upgradeState.error}
          <div class="unavailable">
            <span class="icon-lg">{@html UI_ICONS.alert}</span>
            <div>
              <div class="unavailable-title">Could not generate upgrade recommendations</div>
              <div class="unavailable-sub">{upgradeState.errorDetail}</div>
            </div>
          </div>
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
    height: 100%;
    overflow: hidden; /* window is fixed-size (see tauri.conf.json); main scrolls internally if needed */
    background: var(--bg);
  }
  :global(body) {
    font-family: var(--font-display);
    color: var(--text);
    background-image: radial-gradient(circle, rgba(0, 0, 0, 0.08) 1px, transparent 1px);
    background-size: 24px 24px;
  }

  main {
    /* Hardcoded to the window size in src-tauri/tauri.conf.json
       (app.windows[0].width/height: 525x647, resizable: false) —
       this is the one and only size the window will ever be, on
       every machine, so the layout is built to fit it directly
       instead of being measured and scaled at runtime. */
    position: relative;
    width: 525px;
    height: 647px;
    box-sizing: border-box;
    overflow: hidden; /* keeps the fades/thumb from bleeding past the window edge */
  }

  /* the actual scrolling viewport, filling main edge-to-edge; the
     16px padding here is what leaves the small gap between the
     window edge and the cards */
  .scroll-area {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    padding: 16px;
    overflow-y: auto;
    overflow-x: hidden;
    /* hide the native scrollbar — replaced by .scroll-thumb below */
    scrollbar-width: none; /* Firefox */
    -ms-overflow-style: none; /* old Edge/IE */
  }
  .scroll-area::-webkit-scrollbar {
    width: 0;
    height: 0;
  }

  /* soft gradient hinting that content continues past the edge;
     hidden once you've scrolled all the way to that end */
  .edge-fade {
    position: absolute;
    left: 0;
    right: 0;
    height: 68px;
    pointer-events: none;
    transition: opacity 0.25s ease;
    opacity: 1;
    z-index: 5;
  }
  .edge-fade-top {
    top: 0;
    background: linear-gradient(to bottom, var(--bg) 0%, rgba(255, 255, 255, 0) 100%);
  }
  .edge-fade-bottom {
    bottom: 0;
    background: linear-gradient(to top, var(--bg) 0%, rgba(255, 255, 255, 0) 100%);
  }
  .edge-fade.hidden {
    opacity: 0;
  }

  /* custom scrollbar: one small pill, purely a position indicator (not
     draggable) — invisible at rest, fades in while scrolling or moving
     the mouse over the window, then fades back out shortly after */
  .scroll-thumb {
    position: absolute;
    right: 4px;
    width: 4px;
    border-radius: 3px;
    background: var(--border2);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.3s ease;
    z-index: 10;
  }
  .scroll-thumb.visible {
    opacity: 0.55;
  }

  /* orange circular "scroll down" affordance — sits bottom-right inside
     the fade, disappears once the user reaches the end of the content */
  .scroll-down-btn {
    position: absolute;
    left: 50%;
    bottom: 14px;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: none;
    background: var(--accent);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 3px 10px rgba(255, 107, 0, 0.35);
    opacity: 1;
    transform: translateX(-50%) scale(1) translateY(0);
    transition: opacity 0.25s ease, transform 0.2s ease, background 0.15s ease, box-shadow 0.15s ease;
    z-index: 15;
  }
  .scroll-down-btn svg {
    width: 15px;
    height: 15px;
  }
  .scroll-down-btn:hover {
    background: var(--accent2);
    box-shadow: 0 4px 14px rgba(255, 107, 0, 0.45);
    transform: translateX(-50%) scale(1.06) translateY(0);
  }
  .scroll-down-btn:active {
    transform: translateX(-50%) scale(0.96) translateY(0);
  }
  .scroll-down-btn.hidden {
    opacity: 0;
    pointer-events: none;
    transform: translateX(-50%) scale(0.85) translateY(4px);
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
    /* Fill main's available content width (525px window − 16px padding on
       each side, see the `main` rule above) rather than a separate
       hardcoded number that has to be kept in sync by hand. */
    width: 100%;
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

  /* app-picker modal benchmark run: a pulsing spark */
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
  .battery-panel .panel-header {
    justify-content: space-between;
  }
  .battery-refresh {
    margin-top: 0;
  }
  .battery-idle {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  .battery-none {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text3);
  }
  .battery-caps {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 10px;
    margin-bottom: 14px;
  }
  .battery-cap {
    background: #fff;
    border: 1px solid var(--border);
    border-radius: var(--r2);
    padding: 10px 12px;
  }
  .battery-cap-label {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text3);
    margin-bottom: 4px;
  }
  .battery-cap-value {
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
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

  /* ── Upgrade Button & Modal Styles ── */
  .panel-bottom-action {
    margin-top: 14px;
  }

  .upgrade-pc-btn {
    width: 100%;
    padding: 11px 16px;
    background: #111111;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .upgrade-pc-btn:hover {
    background: var(--accent);
  }

  .upgrade-pc-btn:active {
    transform: scale(0.98);
  }

  .battery-upgrade-action {
    margin-top: 10px;
  }

  .upgrade-battery-btn {
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--text);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .upgrade-battery-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: rgba(255, 107, 0, 0.04);
  }

  .back-link-btn {
    background: none;
    border: none;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    cursor: pointer;
    padding: 0;
  }

  .back-link-btn:hover {
    text-decoration: underline;
  }

  .modal-category-header {
    margin-bottom: 14px;
  }

  .modal-cat-title {
    font-size: 16px;
    font-weight: 700;
    color: var(--text);
    margin-top: 2px;
  }

  /* Pulse Box Container */
  .pulse-box {
    position: relative;
    background: linear-gradient(135deg, rgba(255, 107, 0, 0.05) 0%, rgba(255, 144, 64, 0.1) 100%);
    border: 1.5px solid rgba(255, 107, 0, 0.4);
    border-radius: 8px;
    padding: 12px 14px;
    margin-bottom: 14px;
    animation: slow-pulse 2.8s infinite ease-in-out;
  }

  @keyframes slow-pulse {
    0%, 100% {
      border-color: rgba(255, 107, 0, 0.3);
      box-shadow: 0 0 0 0 rgba(255, 107, 0, 0);
    }
    50% {
      border-color: rgba(255, 107, 0, 0.85);
      box-shadow: 0 0 16px 2px rgba(255, 107, 0, 0.25);
    }
  }

  .pulse-inner {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .pulse-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text);
  }

  .pulse-sub {
    font-size: 11px;
    color: var(--text2);
    margin-top: 2px;
  }

  .modal-bar-list {
    margin-bottom: 16px;
  }

  .bar-row-interactive {
    width: 100%;
    text-align: left;
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 10px;
    cursor: pointer;
    transition: all 0.15s ease;
    margin-bottom: 6px;
  }

  .bar-row-interactive:hover {
    border-color: var(--accent);
    background: #fff9f4;
  }

  .bar-top-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .select-arrow {
    font-size: 11px;
    color: var(--text3);
    transition: color 0.15s ease;
  }

  .bar-row-interactive:hover .select-arrow {
    color: var(--accent);
  }

  .upgrade-loading-modal {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 36px 16px;
    text-align: center;
  }

  .analyzing-spinner {
    color: var(--accent);
    margin-bottom: 12px;
    animation: spin 2s infinite linear;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .upgrade-summary-box {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    background: rgba(255, 107, 0, 0.06);
    border-left: 3px solid var(--accent);
    padding: 10px 12px;
    border-radius: 4px;
    font-size: 12px;
    line-height: 1.45;
    color: var(--text);
    margin-bottom: 14px;
  }

  .lightbulb-icon {
    color: var(--accent);
    display: flex;
    align-items: center;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .upgrade-options-title {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text2);
    margin-bottom: 10px;
  }

  .upgrade-cards-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .upgrade-card-btn {
    width: 100%;
    text-align: left;
    background: #ffffff;
    border: 1.5px solid var(--border);
    border-radius: 8px;
    padding: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    flex-direction: column;
    gap: 6px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.03);
  }

  .upgrade-card-btn:hover {
    border-color: var(--accent);
    background: #fffaf5;
  }

  .upgrade-card-btn.selected {
    border-color: var(--accent);
    background: #fff8f2;
    box-shadow: 0 4px 12px rgba(255, 107, 0, 0.14);
  }

  .card-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .custom-checkbox {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    border: 1.5px solid var(--border2);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 800;
    color: #ffffff;
    background: #ffffff;
    transition: all 0.15s ease;
  }

  .custom-checkbox.checked {
    background: var(--accent);
    border-color: var(--accent);
  }

  .comp-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg3);
    color: var(--text);
    text-transform: uppercase;
  }

  .comp-badge.ram { background: #e0f2fe; color: #0369a1; }
  .comp-badge.storage { background: #dcfce7; color: #15803d; }
  .comp-badge.battery { background: #fef3c7; color: #b45309; }

  .boost-badge {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    color: var(--green);
    background: rgba(22, 163, 74, 0.1);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .card-item-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text);
  }

  .card-item-reason {
    font-size: 11px;
    color: var(--text2);
    line-height: 1.4;
  }

  .proceed-footer {
    margin-top: 18px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .proceed-btn {
    width: 100%;
    padding: 12px 16px;
    background: var(--accent);
    color: #ffffff;
    border: none;
    border-radius: 6px;
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.15s ease, background 0.15s ease, transform 0.1s ease;
  }

  .proceed-btn:hover:not(:disabled) {
    background: #e66000;
    transform: translateY(-1px);
  }

  .proceed-btn:disabled {
    background: #e5e5e5;
    color: #999999;
    cursor: not-allowed;
    opacity: 0.7;
  }
</style>