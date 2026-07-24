<!-- src/routes/redirect/+page.svelte -->
<script>
  import { onMount } from 'svelte'
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import logo from '../../assets/logo.png'

  let upgradeItem = 'Upgrade Hardware'
  let category = 'General'

  $: {
    if ($page.url.searchParams.has('item')) {
      upgradeItem = $page.url.searchParams.get('item')
    }
    if ($page.url.searchParams.has('category')) {
      category = $page.url.searchParams.get('category')
    }
  }

  function goBack() {
    goto('/mainDash')
  }
</script>

<svelte:head>
  <title>Redirecting to MakeDo...</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link
    href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<main>
  <div class="redirect-card">
    <div class="header">
      <img src={logo} alt="MakeDo logo" class="logo" />
      <span class="brand-tag">MAKEDO MARKETPLACE</span>
    </div>

    <div class="spinner-container">
      <div class="pulse-ring"></div>
      <div class="spark-icon">⚡</div>
    </div>

    <h1 class="title">Redirecting to MakeDo site...</h1>
    <p class="subtitle">Finding recommended components for your machine</p>

    <div class="details-box">
      <div class="detail-row">
        <span class="detail-label">Requested Upgrade</span>
        <span class="detail-val highlight">{upgradeItem}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Target Standard</span>
        <span class="detail-val">{category}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Status</span>
        <span class="status-badge">
          <span class="dot"></span> Dummy page — Marketplace integration coming soon
        </span>
      </div>
    </div>

    <div class="actions">
      <button class="back-btn" on:click={goBack}>
        ← Back to Dashboard
      </button>
    </div>
  </div>
</main>

<style>
  :global(:root) {
    --bg: #ffffff;
    --accent: #ff6b00;
    --accent2: #ff9040;
    --text: #111111;
    --text2: #555555;
    --border: #e0e0e0;
    --font-mono: 'JetBrains Mono', monospace;
    --font-display: 'Space Grotesk', sans-serif;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: var(--font-display);
    color: var(--text);
    background: var(--bg);
    background-image: radial-gradient(circle, rgba(0, 0, 0, 0.08) 1px, transparent 1px);
    background-size: 24px 24px;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  main {
    width: 525px;
    height: 647px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    box-sizing: border-box;
  }

  .redirect-card {
    width: 100%;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 32px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.06);
    position: relative;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 24px;
  }

  .logo {
    height: 24px;
    width: auto;
  }

  .brand-tag {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: var(--accent);
    background: rgba(255, 107, 0, 0.08);
    padding: 3px 8px;
    border-radius: 4px;
  }

  .spinner-container {
    position: relative;
    width: 64px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 20px;
  }

  .pulse-ring {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 2px solid var(--accent);
    animation: pulse 1.8s infinite cubic-bezier(0.4, 0, 0.6, 1);
  }

  @keyframes pulse {
    0% {
      transform: scale(0.85);
      opacity: 0.9;
      box-shadow: 0 0 0 0 rgba(255, 107, 0, 0.4);
    }
    70% {
      transform: scale(1.15);
      opacity: 0.2;
      box-shadow: 0 0 0 14px rgba(255, 107, 0, 0);
    }
    100% {
      transform: scale(0.85);
      opacity: 0.9;
      box-shadow: 0 0 0 0 rgba(255, 107, 0, 0);
    }
  }

  .spark-icon {
    font-size: 28px;
  }

  .title {
    font-size: 20px;
    font-weight: 700;
    margin: 0 0 6px 0;
    color: var(--text);
  }

  .subtitle {
    font-size: 13px;
    color: var(--text2);
    margin: 0 0 24px 0;
  }

  .details-box {
    width: 100%;
    background: #f9f9f9;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 28px;
    text-align: left;
  }

  .detail-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .detail-label {
    font-size: 11px;
    font-family: var(--font-mono);
    text-transform: uppercase;
    color: #888;
    letter-spacing: 0.05em;
  }

  .detail-val {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .detail-val.highlight {
    color: var(--accent);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text2);
  }

  .dot {
    width: 7px;
    height: 7px;
    background: #22c55e;
    border-radius: 50%;
    animation: blink 1.2s infinite ease-in-out;
  }

  @keyframes blink {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 1; }
  }

  .actions {
    width: 100%;
  }

  .back-btn {
    width: 100%;
    padding: 12px 18px;
    background: #111111;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.1s ease;
  }

  .back-btn:hover {
    background: var(--accent);
  }

  .back-btn:active {
    transform: scale(0.98);
  }
</style>
