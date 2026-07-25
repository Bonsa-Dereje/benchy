<!-- src/routes/redirect/+page.svelte -->
<script>
  import { onMount, onDestroy } from 'svelte'
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import logo from '../../assets/logo.png'

  let upgradeItems = []
  let category = 'PC performance'

  $: {
    if ($page.url.searchParams.has('item')) {
      const raw = $page.url.searchParams.get('item')
      upgradeItems = raw.split('|').filter(Boolean)
    } else {
      upgradeItems = ['Hardware Upgrade']
    }
    if ($page.url.searchParams.has('category')) {
      category = $page.url.searchParams.get('category')
    }
  }

  // ── Order JSON ──
  // Build a structured order object. The website will read this and flip isRead → true.
  let orderId = ''
  let orderJson = {}
  let isRead = false
  let pollInterval = null
  let jsonStr = ''
  let showReadConfirm = false

  function generateOrderId() {
    const ts = Date.now().toString(36).toUpperCase()
    const rand = Math.random().toString(36).slice(2, 6).toUpperCase()
    return `MKDO-${ts}-${rand}`
  }

  function buildOrder() {
    orderId = generateOrderId()
    orderJson = {
      orderId,
      source: 'benchy-desktop',
      timestamp: new Date().toISOString(),
      customer: {
        device: navigator.userAgent.split(')')[0].split('(')[1] || 'Windows PC',
        os: 'Windows'
      },
      order: {
        category,
        items: upgradeItems.map((title, idx) => ({
          lineItem: idx + 1,
          title,
          type: guessComponent(title),
          qty: 1,
          status: 'pending'
        })),
        totalItems: upgradeItems.length
      },
      isRead: false
    }
    jsonStr = JSON.stringify(orderJson, null, 2)
  }

  function guessComponent(title) {
    const t = title.toLowerCase()
    if (t.includes('ram') || t.includes('memory')) return 'RAM'
    if (t.includes('ssd') || t.includes('storage') || t.includes('nvme')) return 'Storage'
    if (t.includes('battery')) return 'Battery'
    return 'Hardware'
  }

  // ── Poll every 5 s to simulate the site flipping isRead → true ──
  // In a real integration the site writes back to a shared endpoint;
  // here we simulate a confirmation arriving after ~12 s.
  let pollCount = 0
  const SIMULATE_CONFIRM_AFTER = 12000 // ms — remove when real API is wired

  function startPolling() {
    const simulateTimer = setTimeout(() => {
      // Simulate the website reading the order and setting isRead = true
      orderJson = { ...orderJson, isRead: true }
      jsonStr = JSON.stringify(orderJson, null, 2)
    }, SIMULATE_CONFIRM_AFTER)

    pollInterval = setInterval(() => {
      pollCount++
      // Check the (simulated) value
      if (orderJson.isRead) {
        clearInterval(pollInterval)
        clearTimeout(simulateTimer)
        showReadConfirm = true
      }
    }, 5000)

    return simulateTimer
  }

  let simulateTimer = null

  onMount(() => {
    buildOrder()
    simulateTimer = startPolling()
  })

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval)
    if (simulateTimer) clearTimeout(simulateTimer)
  })

  function goBack() {
    goto('/mainDash')
  }
</script>

<svelte:head>
  <title>Order Placed — MakeDo</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link
    href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<main>
  <div class="card">

    <!-- Header -->
    <div class="header">
      <img src={logo} alt="MakeDo logo" class="logo" />
      <span class="brand-tag">MAKEDO MARKETPLACE</span>
    </div>

    <!-- Status area -->
    <div class="status-area">
      {#if showReadConfirm}
        <!-- Confirmed state -->
        <div class="check-ring confirmed">
          <svg viewBox="0 0 24 24" width="26" height="26" fill="none" stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="4 13 9.5 18.5 20 6" />
          </svg>
        </div>
        <h1 class="title">Order Confirmed!</h1>
        <p class="subtitle confirmed-sub">MakeDo has received your order.</p>
      {:else}
        <!-- Waiting state -->
        <div class="check-ring pending">
          <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20 6L9 17l-5-5" />
          </svg>
        </div>
        <h1 class="title">Order Placed</h1>
        <div class="waiting-row">
          <span class="waiting-label">Waiting for confirmation</span>
          <!-- Apple-style bouncing typing dots -->
          <div class="typing-dots" aria-label="Waiting for confirmation">
            <span class="dot-b"></span>
            <span class="dot-b"></span>
            <span class="dot-b"></span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Order items summary -->
    <div class="items-summary">
      {#each upgradeItems as item, i}
        <div class="item-row">
          <span class="item-num">{String(i + 1).padStart(2, '0')}</span>
          <span class="item-title">{item}</span>
          <span class="item-status {showReadConfirm ? 'confirmed' : 'pending'}">
            {showReadConfirm ? 'confirmed' : 'pending'}
          </span>
        </div>
      {/each}
    </div>

    <!-- JSON background panel -->
    <div class="json-panel">
      <div class="json-panel-header">
        <span class="json-label">ORDER PAYLOAD</span>
        <span class="json-badge {orderJson.isRead ? 'read' : 'unread'}">
          isRead: {orderJson.isRead ? 'true' : 'false'}
        </span>
      </div>
      <pre class="json-block"><code>{jsonStr}</code></pre>
      <div class="poll-ticker">
        <span class="poll-dot"></span>
        Polling every 5s · check #{pollCount}
      </div>
    </div>

    <!-- Back button -->
    <button class="back-btn" on:click={goBack}>← Back to Dashboard</button>
  </div>
</main>

<style>
  :global(:root) {
    --bg: #ffffff;
    --accent: #ff6b00;
    --green: #16a34a;
    --text: #111111;
    --text2: #555555;
    --text3: #999999;
    --border: #e0e0e0;
    --font-mono: 'JetBrains Mono', monospace;
    --font-display: 'Space Grotesk', sans-serif;
  }

  :global(*, *::before, *::after) { box-sizing: border-box; }

  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    background: var(--bg);
    background-image: radial-gradient(circle, rgba(0,0,0,0.07) 1px, transparent 1px);
    background-size: 24px 24px;
  }

  :global(body) {
    font-family: var(--font-display);
    color: var(--text);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  main {
    width: 525px;
    height: 647px;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 14px;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: none;
  }
  main::-webkit-scrollbar { display: none; }

  /* ── Card ── */
  .card {
    width: 100%;
    background: #fff;
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 22px 20px 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 10px 28px rgba(0,0,0,0.06);
  }

  /* ── Header ── */
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .logo { height: 20px; width: auto; }
  .brand-tag {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--accent);
    background: rgba(255,107,0,0.08);
    padding: 3px 7px;
    border-radius: 4px;
  }

  /* ── Status area ── */
  .status-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 10px 0 4px;
  }

  .check-ring {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .check-ring.pending {
    background: var(--accent);
    box-shadow: 0 0 0 6px rgba(255,107,0,0.12);
    animation: ring-pulse 2s infinite ease-in-out;
  }
  .check-ring.confirmed {
    background: var(--green);
    box-shadow: 0 0 0 6px rgba(22,163,74,0.14);
    animation: pop-in 0.35s cubic-bezier(0.34,1.56,0.64,1);
  }

  @keyframes ring-pulse {
    0%, 100% { box-shadow: 0 0 0 6px rgba(255,107,0,0.12); }
    50%       { box-shadow: 0 0 0 10px rgba(255,107,0,0.06); }
  }
  @keyframes pop-in {
    from { transform: scale(0.6); opacity: 0; }
    to   { transform: scale(1);   opacity: 1; }
  }

  .title {
    font-size: 20px;
    font-weight: 700;
    margin: 0;
    color: var(--text);
  }

  /* Waiting-for-confirmation row */
  .waiting-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .waiting-label {
    font-size: 12px;
    color: var(--text2);
  }
  .confirmed-sub {
    font-size: 13px;
    color: var(--green);
    font-weight: 600;
    margin: 0;
  }

  /* ── Typing / bouncing dots (Apple iMessage style) ── */
  .typing-dots {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    height: 16px;
  }
  .dot-b {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text3);
    animation: bounce-dot 1.2s infinite ease-in-out;
  }
  .dot-b:nth-child(1) { animation-delay: 0s; }
  .dot-b:nth-child(2) { animation-delay: 0.18s; }
  .dot-b:nth-child(3) { animation-delay: 0.36s; }

  @keyframes bounce-dot {
    0%, 80%, 100% { transform: translateY(0);    opacity: 0.35; }
    40%           { transform: translateY(-6px);  opacity: 1; }
  }

  /* ── Items list ── */
  .items-summary {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .item-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    background: #fafafa;
    border: 1px solid var(--border);
    border-radius: 7px;
  }
  .item-num {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    color: var(--text3);
    min-width: 22px;
  }
  .item-title {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }
  .item-status {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 2px 7px;
    border-radius: 4px;
  }
  .item-status.pending   { background: rgba(255,107,0,0.1);  color: var(--accent); }
  .item-status.confirmed { background: rgba(22,163,74,0.1);  color: var(--green); }

  /* ── JSON panel ── */
  .json-panel {
    background: #f4f4f5;
    border: 1px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
  }
  .json-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid #e2e2e3;
    background: #ececed;
  }
  .json-label {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.07em;
    color: #555;
  }
  .json-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    padding: 2px 7px;
    border-radius: 4px;
    transition: background 0.3s, color 0.3s;
  }
  .json-badge.unread { background: rgba(255,107,0,0.12); color: var(--accent); }
  .json-badge.read   { background: rgba(22,163,74,0.12);  color: var(--green); animation: badge-pop 0.4s ease; }

  @keyframes badge-pop {
    0%   { transform: scale(0.8); }
    60%  { transform: scale(1.15); }
    100% { transform: scale(1); }
  }

  .json-block {
    margin: 0;
    padding: 10px 12px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    line-height: 1.6;
    color: #333;
    overflow-x: auto;
    max-height: 180px;
    overflow-y: auto;
    white-space: pre;
    scrollbar-width: thin;
    scrollbar-color: #ccc transparent;
  }

  .poll-ticker {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-top: 1px solid #e2e2e3;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text3);
  }
  .poll-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    animation: blink-dot 1s infinite ease-in-out;
  }
  @keyframes blink-dot {
    0%, 100% { opacity: 0.25; }
    50%       { opacity: 1; }
  }

  /* ── Back button ── */
  .back-btn {
    width: 100%;
    padding: 11px 16px;
    background: #111;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }
  .back-btn:hover  { background: var(--accent); }
  .back-btn:active { transform: scale(0.98); }
</style>
