// Reports the active tab's URL to Korio over loopback. Korio reduces it to a
// registrable domain; full URLs never leave the machine and are never stored.

async function config() {
  const { token = "", port = 7878 } = await chrome.storage.local.get(["token", "port"]);
  return { token, port };
}

async function report(url) {
  const { token, port } = await config();
  if (!token) return null; // not paired yet
  try {
    const r = await fetch(`http://127.0.0.1:${port}/active`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ token, url: url || null }),
    });
    if (!r.ok) return null;
    const ct = r.headers.get("content-type") || "";
    return ct.includes("application/json") ? await r.json() : null;
  } catch (_) { return null; } // Korio not running / feature off — ignore
}

async function reportActiveTab() {
  try {
    const [tab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
    const url = tab && /^https?:/.test(tab.url || "") ? tab.url : null;
    const res = await report(url);
    if (res && res.blocked && tab && tab.id != null) {
      // Domain is over its Auto-close cap → close this tab (never the whole browser).
      try { await chrome.tabs.remove(tab.id); } catch (_) {}
    }
  } catch (_) { await report(null); }
}

chrome.tabs.onActivated.addListener(reportActiveTab);
chrome.tabs.onUpdated.addListener((_id, info) => { if (info.url || info.status === "complete") reportActiveTab(); });
chrome.windows.onFocusChanged.addListener((winId) => {
  if (winId === chrome.windows.WINDOW_ID_NONE) report(null); // browser lost focus
  else reportActiveTab();
});

// Heartbeat: re-report the active tab every 30s so Korio's ActiveSite stays fresh
// while the user reads a static page (no tab/focus events fire then). chrome.alarms
// wakes the MV3 service worker even after it has been suspended.
chrome.alarms.create("korio-heartbeat", { periodInMinutes: 0.5 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "korio-heartbeat") reportActiveTab();
});
