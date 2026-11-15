/* ═══════════════════════════════════════════════════════════════════════════
   EchoGrid — Frontend logic (v1.4.0 — icônes SVG N&B, UX améliorée)
   ═══════════════════════════════════════════════════════════════════════════ */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ── SVG fragments (noir & blanc) ────────────────────────────────────────────
const SVG = {
  cross:   '<svg class="icon" viewBox="0 0 20 20"><path d="M5 5l10 10M15 5L5 15" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round"/></svg>',
  check:   '<svg class="icon" viewBox="0 0 20 20"><path d="M3 10l4 4 10-10" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>',
  x:       '<svg class="icon-sm" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round"/></svg>',
  up:      '<svg class="icon-sm" viewBox="0 0 16 16"><path d="M8 2l6 10H2z" fill="currentColor"/></svg>',
  down:    '<svg class="icon-sm" viewBox="0 0 16 16"><path d="M8 14L2 4h12z" fill="currentColor"/></svg>',
  clock:   '<svg class="icon-sm" viewBox="0 0 16 16"><circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M8 4.5V8l2.5 1.5" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>',
  jitter:  '<svg class="icon-sm" viewBox="0 0 16 16"><path d="M1 8h3l2-5 3 10 2-5h4" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>',
  minmax:  '<svg class="icon-sm" viewBox="0 0 16 16"><path d="M2 12l3-6 3 3 3-5h3" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d="M14 7l1 1-1 1" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>',
  lost:    '<svg class="icon-sm" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round"/></svg>',
  ok:      '<svg class="icon-sm" viewBox="0 0 16 16"><path d="M3 8l3 3 7-7" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>',
  stop:    '<svg class="icon" viewBox="0 0 20 20"><rect x="4" y="4" width="12" height="12" rx="2" fill="currentColor"/></svg>',
};

// ── Notification API (Tauri plugin) ──────────────────────────────────────────
let sendNotification = null;
let notificationReady = false;

(async () => {
  try {
    const notif = window.__TAURI__.notification;
    if (notif) {
      let granted = await notif.isPermissionGranted();
      if (!granted) {
        const perm = await notif.requestPermission();
        granted = perm === 'granted';
      }
      if (granted) {
        sendNotification = notif.sendNotification;
        notificationReady = true;
      }
    }
  } catch (e) {
    console.warn('Notification plugin not available:', e);
  }
})();

// ── Window focus tracking ────────────────────────────────────────────────────
let windowFocused = true;

document.addEventListener('visibilitychange', () => {
  windowFocused = !document.hidden;
});
window.addEventListener('focus', () => { windowFocused = true; });
window.addEventListener('blur',  () => { windowFocused = false; });

// ── Alert sound ──────────────────────────────────────────────────────────────
let alertAudioCtx = null;

function playAlertSound() {
  try {
    if (!alertAudioCtx) {
      alertAudioCtx = new (window.AudioContext || window.webkitAudioContext)();
    }
    const osc = alertAudioCtx.createOscillator();
    const gain = alertAudioCtx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(880, alertAudioCtx.currentTime);
    osc.frequency.setValueAtTime(440, alertAudioCtx.currentTime + 0.15);
    gain.gain.setValueAtTime(0.3, alertAudioCtx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, alertAudioCtx.currentTime + 0.4);
    osc.connect(gain);
    gain.connect(alertAudioCtx.destination);
    osc.start(alertAudioCtx.currentTime);
    osc.stop(alertAudioCtx.currentTime + 0.4);
  } catch (e) {
    console.warn('Alert sound error:', e);
  }
}

// ── Toast notifications ──────────────────────────────────────────────────────

const toastContainer = document.getElementById('toast-container');

function showToast(message, type = 'error', duration = 4000) {
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  // Icon
  if (type === 'error') {
    toast.innerHTML = SVG.x + `<span>${message}</span>`;
  } else {
    toast.innerHTML = SVG.check + `<span>${message}</span>`;
  }
  toastContainer.appendChild(toast);
  setTimeout(() => {
    toast.style.opacity = '0';
    toast.style.transition = 'opacity 0.3s';
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

// ── DOM refs ─────────────────────────────────────────────────────────────────

const hostsGrid    = document.getElementById('hosts-grid');
const emptyState   = document.getElementById('empty-state');
const hostInput    = document.getElementById('host-input');
const portInput    = document.getElementById('port-input');
const monitorType  = document.getElementById('monitor-type');
const addBtn       = document.getElementById('add-btn');
const stopAllBtn   = document.getElementById('stop-all-btn');
const settingsBtn  = document.getElementById('settings-btn');
const settingsModal = document.getElementById('settings-modal');
const historyBtn   = document.getElementById('history-btn');
const historyPanel = document.getElementById('history-panel');
const historyClose = document.getElementById('history-close');
const historyBody  = document.getElementById('history-body');
const historyFilter = document.getElementById('history-filter');

// ── Confirm modal ────────────────────────────────────────────────────────────

const confirmModal = document.getElementById('confirm-modal');
const confirmMsg   = document.getElementById('confirm-msg');
const confirmYes   = document.getElementById('confirm-yes');
const confirmNo    = document.getElementById('confirm-no');

let confirmResolve = null;

function showConfirm(msg) {
  return new Promise(resolve => {
    confirmResolve = resolve;
    confirmMsg.textContent = msg;
    confirmModal.classList.remove('hidden');
  });
}

confirmYes.addEventListener('click', () => {
  confirmModal.classList.add('hidden');
  if (confirmResolve) confirmResolve(true);
  confirmResolve = null;
});

confirmNo.addEventListener('click', () => {
  confirmModal.classList.add('hidden');
  if (confirmResolve) confirmResolve(false);
  confirmResolve = null;
});

// ── State ────────────────────────────────────────────────────────────────────

const hosts = new Map(); // key → { type, tile, alive, wasAlive, total_count, lost_count }
const history = [];      // { time, host, state, latency, detail }
const MAX_HISTORY = 500;
let settings = {
  ping_interval_ms: 2000,
  ping_timeout_ms: 1000,
  ttl: 64,
  packet_size: 32,
  tcp_timeout_ms: 3000,
  alert_sound: true,
  latency_warn_ms: 50.0,
  latency_crit_ms: 150.0,
};

// ── Empty state ─────────────────────────────────────────────────────────────

function toggleEmptyState() {
  if (emptyState) {
    emptyState.classList.toggle('hidden', hosts.size > 0);
  }
}

// ── Hosts persistence (localStorage) ─────────────────────────────────────────

const STORAGE_KEY = 'echogrid-hosts';

function saveHostsToStorage() {
  const list = [];
  for (const [key, entry] of hosts.entries()) {
    if (key.includes(':')) {
      const [host, port] = key.split(':');
      list.push({ type: 'tcp', host, port: parseInt(port) });
    } else {
      list.push({ type: 'icmp', host: key });
    }
  }
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
  } catch (e) {
    console.warn('localStorage save error:', e);
  }
}

function loadHostsFromStorage() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    return JSON.parse(raw);
  } catch (e) {
    console.warn('localStorage load error:', e);
    return [];
  }
}

// ── History panel ─────────────────────────────────────────────────────────────

historyBtn.addEventListener('click', () => historyPanel.classList.toggle('hidden'));
historyClose.addEventListener('click', () => historyPanel.classList.add('hidden'));

function addHistory(host, state, latency, detail) {
  const now = new Date();
  const time = now.toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  history.unshift({ time, host, state, latency, detail });
  if (history.length > MAX_HISTORY) history.length = MAX_HISTORY;
  renderHistory();
}

function renderHistory() {
  const filter = historyFilter.value;
  const fragment = document.createDocumentFragment();
  for (const h of history) {
    if (filter !== 'all') {
      const map = { up: 'UP', down: 'DOWN', error: 'ERR' };
      if (h.state !== map[filter]) continue;
    }
    const tr = document.createElement('tr');
    const stateClass = h.state === 'UP' ? 'history-up' : h.state === 'DOWN' ? 'history-down' : 'history-error';
    tr.innerHTML =
      `<td>${h.time}</td>` +
      `<td>${h.host}</td>` +
      `<td class="${stateClass}">${h.state}</td>` +
      `<td>${h.latency}</td>` +
      `<td style="color:var(--text-dim)">${h.detail || ''}</td>`;
    fragment.appendChild(tr);
  }
  historyBody.innerHTML = '';
  historyBody.appendChild(fragment);
}

historyFilter.addEventListener('change', renderHistory);

function updateStats() {
  let total = 0, up = 0, down = 0;
  let totalPings = 0, totalLost = 0;
  for (const entry of hosts.values()) {
    total++;
    if (entry.alive) up++; else down++;
    totalPings += (entry.total_count || 0);
    totalLost += (entry.lost_count || 0);
  }
  const totalEl = document.getElementById('stats-total');
  const upEl    = document.getElementById('stats-up');
  const downEl  = document.getElementById('stats-down');
  const upEl2   = document.getElementById('stats-uptime');

  // Update only the number part (second child = <span>)
  totalEl.querySelector('span').textContent = total;
  upEl.querySelector('span').textContent    = up;
  downEl.querySelector('span').textContent  = down;

  // Uptime %
  if (totalPings > 0) {
    const uptimePct = ((totalPings - totalLost) / totalPings * 100).toFixed(1);
    upEl2.querySelector('span').textContent = `${uptimePct}%`;
  } else {
    upEl2.querySelector('span').textContent = '—';
  }
}

function notifyKO(host, error) {
  if (!notificationReady || !sendNotification) return;
  if (windowFocused) return;
  try {
    sendNotification({
      title: 'EchoGrid — Hôte KO',
      body: error ? `${host} : ${error}` : `${host} est DOWN`,
    });
  } catch (e) {
    console.warn('Notification error:', e);
  }
}

// ── last_ok formatting (short: HH:MM:SS only) ───────────────────────────────

function formatLastOk(isoStr) {
  if (!isoStr) return null;
  const parts = isoStr.split(' ');
  if (parts.length >= 2) return parts[1];
  return isoStr;
}

// ── Add host ─────────────────────────────────────────────────────────────────

addBtn.addEventListener('click', addHost);
hostInput.addEventListener('keydown', e => { if (e.key === 'Enter') addHost(); });

monitorType.addEventListener('change', () => {
  portInput.style.display = monitorType.value === 'tcp' ? 'inline' : 'none';
  if (monitorType.value === 'tcp') portInput.focus();
});

async function addHost() {
  const host = hostInput.value.trim();
  if (!host) return;

  const type = monitorType.value;

  try {
    if (type === 'icmp') {
      const key = host;
      if (hosts.has(key)) return;
      createTile(key, host, 'ICMP');
      await invoke('start_ping', {
        config: {
          host,
          interval_ms: settings.ping_interval_ms,
          timeout_ms: settings.ping_timeout_ms,
          ttl: settings.ttl,
          packet_size: settings.packet_size,
        },
      });
    } else {
      const port = parseInt(portInput.value) || 80;
      const key = `${host}:${port}`;
      if (hosts.has(key)) return;
      createTile(key, host, `TCP ${port}`);
      await invoke('start_tcp_monitor', {
        config: {
          host,
          port,
          interval_ms: settings.ping_interval_ms,
          timeout_ms: settings.tcp_timeout_ms,
        },
      });
    }

    hostInput.value = '';
    portInput.value = '';
    hostInput.focus();
    toggleEmptyState();
    updateStats();
    saveHostsToStorage();
  } catch (e) {
    showToast(`${e}`, 'error');
    console.warn('addHost error:', e);
  }
}

// ── Create tile ──────────────────────────────────────────────────────────────

function createTile(key, host, label) {
  const tile = document.createElement('div');
  tile.className = 'host-tile';
  tile.innerHTML = `
    <button class="stop-btn" title="Arrêter ce monitoring">${SVG.cross}</button>
    <span class="host-type">${label}</span>
    <span class="host-name">${host}</span>
    <span class="status-dot"></span>
    <span class="latency">—</span>
    <span class="latency-unit"></span>
    <div class="metrics">
      <span class="metric-lost" title="Pings perdus / total">${SVG.lost} 0</span>
      <span class="metric-last-ok" title="Dernier OK">${SVG.clock} —</span>
      <span class="metric-jitter" title="Jitter">${SVG.jitter} —</span>
      <span class="metric-minmax" title="Min / Max">${SVG.minmax} —</span>
    </div>
  `;

  tile.querySelector('.stop-btn').addEventListener('click', () => stopHost(key));
  hostsGrid.appendChild(tile);
  hosts.set(key, { type: label, tile, alive: false, wasAlive: false, total_count: 0, lost_count: 0 });
  toggleEmptyState();
}

// ── Stop host ────────────────────────────────────────────────────────────────

async function stopHost(key) {
  const entry = hosts.get(key);
  if (!entry) return;
  entry.tile.remove();
  hosts.delete(key);
  toggleEmptyState();
  updateStats();
  saveHostsToStorage();

  try {
    if (key.includes(':')) {
      const [host, port] = key.split(':');
      await invoke('stop_tcp_monitor', { host, port: parseInt(port) });
    } else {
      await invoke('stop_ping', { host: key });
    }
  } catch (e) {
    showToast(`Stop : ${e}`, 'error');
    console.warn('stop error:', e);
  }
}

// ── Stop all ─────────────────────────────────────────────────────────────────

stopAllBtn.addEventListener('click', async () => {
  if (hosts.size === 0) return;
  const confirmed = await showConfirm(`Arrêter les ${hosts.size} monitoring(s) en cours ?`);
  if (!confirmed) return;

  try {
    await invoke('stop_all_pings');
    await invoke('stop_all_tcp_monitors');
  } catch (e) {
    showToast(`Stop-all : ${e}`, 'error');
    console.warn('stop-all error:', e);
  }
  // Remove all tiles
  for (const entry of hosts.values()) entry.tile.remove();
  hosts.clear();
  toggleEmptyState();
  updateStats();
  saveHostsToStorage();
});

// ── Update tile ──────────────────────────────────────────────────────────────

function updateTile(key, alive, latencyMs, error, metrics) {
  const entry = hosts.get(key);
  if (!entry) return;

  const wasAlive = entry.alive;
  entry.alive = alive;

  // Track cumulative counts for uptime
  if (metrics) {
    entry.total_count = metrics.total_count;
    entry.lost_count = metrics.lost_count;
  }

  const tile = entry.tile;
  tile.classList.remove('up', 'down', 'error');

  const latencyEl = tile.querySelector('.latency');
  const unitEl    = tile.querySelector('.latency-unit');

  // Remove previous latency threshold classes
  latencyEl.classList.remove('latency-warn', 'latency-crit');

  if (alive) {
    tile.classList.add('up');
    latencyEl.textContent = latencyMs.toFixed(1);
    unitEl.textContent    = 'ms';

    // Latence colorée par seuil
    if (settings.latency_crit_ms > 0 && latencyMs >= settings.latency_crit_ms) {
      latencyEl.classList.add('latency-crit');
    } else if (settings.latency_warn_ms > 0 && latencyMs >= settings.latency_warn_ms) {
      latencyEl.classList.add('latency-warn');
    }
  } else if (error) {
    tile.classList.add('error');
    latencyEl.textContent = error.length > 18 ? error.substring(0, 18) + '…' : error;
    unitEl.textContent    = '';
  } else {
    tile.classList.add('down');
    latencyEl.textContent = 'DOWN';
    unitEl.textContent    = '';
  }

  // ── Update metrics ──
  if (metrics) {
    const lostEl     = tile.querySelector('.metric-lost');
    const lastOkEl   = tile.querySelector('.metric-last-ok');
    const jitterEl   = tile.querySelector('.metric-jitter');
    const minMaxEl   = tile.querySelector('.metric-minmax');

    if (metrics.lost_count > 0) {
      lostEl.innerHTML = `${SVG.lost} ${metrics.lost_count}/${metrics.total_count}`;
      lostEl.classList.add('has-loss');
    } else {
      lostEl.innerHTML = `${SVG.ok} ${metrics.total_count}`;
      lostEl.classList.remove('has-loss');
    }

    const shortLastOk = formatLastOk(metrics.last_ok);
    lastOkEl.innerHTML = shortLastOk ? `${SVG.clock} ${shortLastOk}` : `${SVG.clock} —`;

    if (metrics.jitter_ms != null) {
      jitterEl.innerHTML = `${SVG.jitter} ${metrics.jitter_ms.toFixed(1)}ms`;
      jitterEl.classList.toggle('high-jitter', metrics.jitter_ms > 50);
    } else {
      jitterEl.innerHTML = `${SVG.jitter} —`;
      jitterEl.classList.remove('high-jitter');
    }

    if (metrics.latency_min != null && metrics.latency_max != null) {
      minMaxEl.innerHTML = `${SVG.minmax} ${metrics.latency_min.toFixed(0)}/${metrics.latency_max.toFixed(0)}ms`;
    } else {
      minMaxEl.innerHTML = `${SVG.minmax} —`;
    }
  }

  // ── Alert sound + notification KO ──
  if (!alive && wasAlive) {
    notifyKO(key, error);
    if (settings.alert_sound) {
      playAlertSound();
    }
  }

  // ── History log ──
  if (alive !== wasAlive || alive) {
    const state = alive ? 'UP' : (error ? 'ERR' : 'DOWN');
    const latencyStr = alive ? `${latencyMs.toFixed(1)} ms` : '—';
    if (alive !== wasAlive || !alive) {
      addHistory(key, state, latencyStr, error || '');
    }
  }

  updateStats();
}

// ── Tauri event listeners ────────────────────────────────────────────────────

listen('ping-result', (event) => {
  const r = event.payload;
  updateTile(r.host, r.alive, r.latency_ms, r.error, {
    lost_count:   r.lost_count,
    total_count:  r.total_count,
    last_ok:      r.last_ok,
    latency_min:  r.latency_min,
    latency_max:  r.latency_max,
    jitter_ms:    r.jitter_ms,
  });
});

listen('tcp-result', (event) => {
  const r = event.payload;
  const key = `${r.host}:${r.port}`;
  updateTile(key, r.open, r.latency_ms, r.error, {
    lost_count:   r.lost_count,
    total_count:  r.total_count,
    last_ok:      r.last_ok,
    latency_min:  r.latency_min,
    latency_max:  r.latency_max,
    jitter_ms:    r.jitter_ms,
  });
});

listen('ping-stopped', (event) => {
  const host = event.payload;
  const entry = hosts.get(host);
  if (entry) { entry.tile.remove(); hosts.delete(host); toggleEmptyState(); updateStats(); saveHostsToStorage(); }
});

listen('tcp-stopped', (event) => {
  const key = event.payload;
  const entry = hosts.get(key);
  if (entry) { entry.tile.remove(); hosts.delete(key); toggleEmptyState(); updateStats(); saveHostsToStorage(); }
});

// ── Settings modal ───────────────────────────────────────────────────────────

settingsBtn.addEventListener('click', () => settingsModal.classList.remove('hidden'));
document.getElementById('settings-close').addEventListener('click', () => settingsModal.classList.add('hidden'));

document.getElementById('settings-save').addEventListener('click', async () => {
  const newSettings = {
    ping_interval_ms: parseInt(document.getElementById('set-interval').value)     || 2000,
    ping_timeout_ms: parseInt(document.getElementById('set-timeout').value)      || 1000,
    ttl:             parseInt(document.getElementById('set-ttl').value)          || 64,
    packet_size:     parseInt(document.getElementById('set-packet-size').value)  || 32,
    tcp_timeout_ms:  parseInt(document.getElementById('set-tcp-timeout').value)  || 3000,
    alert_sound:     document.getElementById('set-alert-sound').checked,
    latency_warn_ms: parseFloat(document.getElementById('set-latency-warn').value) || 50,
    latency_crit_ms: parseFloat(document.getElementById('set-latency-crit').value) || 150,
  };
  try {
    await invoke('save_settings', { settings: newSettings });
    settings = newSettings;
    settingsModal.classList.add('hidden');
    showToast('Paramètres sauvegardés', 'success', 2000);
  } catch (e) {
    showToast(`Settings : ${e}`, 'error');
    console.warn('save_settings error:', e);
  }
});

// ── Load settings on startup ─────────────────────────────────────────────────

(async () => {
  try {
    const s = await invoke('get_settings');
    settings = s;
    document.getElementById('set-interval').value    = s.ping_interval_ms;
    document.getElementById('set-timeout').value     = s.ping_timeout_ms;
    document.getElementById('set-ttl').value         = s.ttl;
    document.getElementById('set-packet-size').value = s.packet_size;
    document.getElementById('set-tcp-timeout').value = s.tcp_timeout_ms;
    document.getElementById('set-alert-sound').checked = s.alert_sound !== false;
    document.getElementById('set-latency-warn').value = s.latency_warn_ms || 50;
    document.getElementById('set-latency-crit').value = s.latency_crit_ms || 150;
  } catch (e) {
    console.warn('could not load settings:', e);
    showToast('Impossible de charger les paramètres', 'error');
  }
})();

// ── Restore persisted hosts on startup ───────────────────────────────────────

(async () => {
  const saved = loadHostsFromStorage();
  for (const item of saved) {
    try {
      if (item.type === 'icmp') {
        const key = item.host;
        if (hosts.has(key)) continue;
        createTile(key, item.host, 'ICMP');
        await invoke('start_ping', {
          config: {
            host: item.host,
            interval_ms: settings.ping_interval_ms,
            timeout_ms: settings.ping_timeout_ms,
            ttl: settings.ttl,
            packet_size: settings.packet_size,
          },
        });
      } else if (item.type === 'tcp') {
        const port = item.port || 80;
        const key = `${item.host}:${port}`;
        if (hosts.has(key)) continue;
        createTile(key, item.host, `TCP ${port}`);
        await invoke('start_tcp_monitor', {
          config: {
            host: item.host,
            port,
            interval_ms: settings.ping_interval_ms,
            timeout_ms: settings.tcp_timeout_ms,
          },
        });
      }
    } catch (e) {
      console.warn(`Could not restore host ${item.host}:`, e);
      showToast(`Hôte ${item.host} non restauré`, 'error');
    }
  }
  toggleEmptyState();
  updateStats();
})();
