// Reports the active tab's URL to Korio over loopback. Korio reduces it to a
// registrable domain; full URLs never leave the machine and are never stored.

async function config() {
  const { token = "", port = 7878 } = await chrome.storage.local.get(["token", "port"]);
  return { token, port };
}

async function report(url) {
  const { token, port } = await config();
  if (!token) return; // not paired yet
  try {
    await fetch(`http://127.0.0.1:${port}/active`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ token, url: url || null }),
    });
  } catch (_) { /* Korio not running / feature off — ignore silently */ }
}

async function reportActiveTab() {
  try {
    const [tab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
    const url = tab && /^https?:/.test(tab.url || "") ? tab.url : null;
    await report(url);
  } catch (_) { await report(null); }
}

chrome.tabs.onActivated.addListener(reportActiveTab);
chrome.tabs.onUpdated.addListener((_id, info) => { if (info.url || info.status === "complete") reportActiveTab(); });
chrome.windows.onFocusChanged.addListener((winId) => {
  if (winId === chrome.windows.WINDOW_ID_NONE) report(null); // browser lost focus
  else reportActiveTab();
});
