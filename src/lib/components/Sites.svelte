<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { siteUsageRange, clearSite, colorFor, listSiteCaps, setSiteLimit,
    browserStatus, type SiteUsage, type SiteCap, type BrowserStatus } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { readJSON, writeJSON } from "$lib/prefs";
  import { navIntent } from "$lib/nav";

  type RangeId = "today" | "7d" | "30d";
  let range = $state<RangeId>("today");
  type Row = { domain: string; seconds: number; cap: number; action: string };
  let rows = $state<Row[]>([]);
  let loading = $state(true);
  let status = $state<BrowserStatus | null>(null);
  let checking = $state(false);
  let connected = $derived(status?.connected === true);
  let statusPoll: ReturnType<typeof setInterval> | null = null;
  async function refreshStatus() {
    checking = true;
    try { status = await browserStatus(); } catch { status = null; }
    checking = false;
  }
  async function recheck() { await refreshStatus(); await load(); }
  function goSettings() { navIntent.set("settings"); }

  function todayStart(): number {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    return Math.floor(d.getTime() / 1000);
  }
  function bounds(r: RangeId): [number, number] {
    const start = todayStart();
    const days = r === "today" ? 1 : r === "7d" ? 7 : 30;
    return [start - (days - 1) * 86400, start + 86400];
  }

  async function load() {
    loading = true;
    const [from, to] = bounds(range);
    let usage: SiteUsage[] = [];
    let caps: SiteCap[] = [];
    try { [usage, caps] = await Promise.all([siteUsageRange(from, to), listSiteCaps()]); } catch { usage = []; caps = []; }
    const capMap = new Map(caps.map((c) => [c.domain, c]));
    const seen = new Set<string>();
    const merged: Row[] = usage.map((u) => {
      seen.add(u.domain);
      const c = capMap.get(u.domain);
      return { domain: u.domain, seconds: u.seconds, cap: c?.daily_cap_seconds ?? 0, action: c?.limit_action ?? "warn" };
    });
    for (const c of caps) {
      if (!seen.has(c.domain) && c.daily_cap_seconds > 0) {
        merged.push({ domain: c.domain, seconds: 0, cap: c.daily_cap_seconds, action: c.limit_action });
      }
    }
    rows = merged;
    loading = false;
  }
  async function setRange(r: RangeId) { range = r; await load(); }
  async function remove(domain: string) { await clearSite(domain); await load(); }
  async function commitCap(row: Row, minutes: number, action: string) {
    await setSiteLimit(row.domain, Math.max(0, Math.round(minutes)) * 60, action);
    await load();
  }

  let minSeconds = $state<number>(readJSON<number>("korio.sites.minSeconds", 0));
  function setMin(v: number) { minSeconds = v; writeJSON("korio.sites.minSeconds", v); }
  // Visible rows: at/above threshold, OR has a configured cap (never hide a limit you set).
  let visible = $derived(rows.filter((r) => r.seconds >= minSeconds || r.cap > 0));
  let visibleTotal = $derived(visible.reduce((a, r) => a + r.seconds, 0));

  let max = $derived(rows.reduce((m, s) => Math.max(m, s.seconds), 0));
  let total = $derived(rows.reduce((a, s) => a + s.seconds, 0));

  onMount(async () => { await refreshStatus(); await load(); statusPoll = setInterval(refreshStatus, 15_000); });
  onDestroy(() => { if (statusPoll) clearInterval(statusPoll); });
</script>

<div class="sites">
  {#if !connected}
    <div class="notconnected">
      <h3>Browser not connected</h3>
      <p>Korio isn't receiving any site data yet. To track the sites you visit:</p>
      <ol>
        <li>Open <strong>Settings</strong> and turn on <strong>Track browser sites</strong>.</li>
        <li>Install the <strong>Korio browser extension</strong> in your browser.</li>
        <li>In the extension's options page, paste the <strong>pairing token</strong> from Settings{#if status}{' '}(port {status.port}){/if}.</li>
        <li>Come back here and press <strong>Refresh</strong>.</li>
      </ol>
      <div class="ncbtns">
        <button class="primary" onclick={goSettings}>Open Settings</button>
        <button class="ghost" onclick={recheck} disabled={checking}>{checking ? "Checking…" : "Refresh"}</button>
      </div>
    </div>
  {:else}
  <div class="bar">
    <div class="ranges">
      <button class:on={range === "today"} onclick={() => setRange("today")}>Today</button>
      <button class:on={range === "7d"} onclick={() => setRange("7d")}>7 days</button>
      <button class:on={range === "30d"} onclick={() => setRange("30d")}>30 days</button>
    </div>
    <div class="barright">
      <label class="minlbl">Hide under
        <select aria-label="Hide sites under" value={String(minSeconds)}
          onchange={(e) => setMin(+e.currentTarget.value)}>
          <option value="0">Off</option>
          <option value="60">1m</option>
          <option value="300">5m</option>
          <option value="600">10m</option>
          <option value="1800">30m</option>
        </select>
      </label>
      <span class="total">{formatDuration(visibleTotal)} total</span>
    </div>
  </div>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if rows.length === 0}
    <p class="empty">No site activity tracked in this range. Enable Browser tracking in Settings and install the Korio extension.</p>
  {:else if visible.length === 0}
    <p class="empty">All tracked sites are under the "hide under" threshold. Lower it to see them.</p>
  {:else}
    <ul>
      {#each visible as s, i (s.domain)}
        <li>
          <span class="name" title={s.domain}>{s.domain}</span>
          <span class="track"><span class="fill" style={`width:${max > 0 ? (s.seconds / max) * 100 : 0}%;background:${colorFor(i)}`}></span></span>
          <span class="t">{formatDuration(s.seconds)}</span>
          <span class="limit">
            <input class="cap" type="number" min="0" step="5" value={Math.round(s.cap / 60)}
              title="Daily limit (minutes, 0 = off)" aria-label={`Daily limit minutes for ${s.domain}`}
              onchange={(e) => commitCap(s, +e.currentTarget.value, s.action)} />
            <button class="act" class:close={s.action === "close"}
              title={s.action === "close" ? "Auto-close tab when over limit" : "Warn when over limit"}
              onclick={() => commitCap(s, Math.round(s.cap / 60), s.action === "close" ? "warn" : "close")}>
              {s.action === "close" ? "Auto-close" : "Warn"}
            </button>
          </span>
          <button class="del" aria-label={`Clear history for ${s.domain}`} onclick={() => remove(s.domain)}>✕</button>
        </li>
      {/each}
    </ul>
  {/if}
  {/if}
</div>

<style>
  .sites { max-width: 720px; }
  .bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 18px; }
  .ranges { display: flex; gap: 6px; }
  .ranges button { padding: 6px 12px; border: 1px solid var(--line); background: var(--surface);
    color: var(--muted); border-radius: var(--radius-sm); font: inherit; font-size: 13px; cursor: pointer; }
  .ranges button.on { color: var(--accent-contrast); background: var(--accent); border-color: var(--accent); }
  .total { font-size: 13px; color: var(--muted); font-variant-numeric: tabular-nums; }
  .empty { color: var(--muted); font-size: 13px; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
  li { display: grid; grid-template-columns: 160px 1fr auto auto auto; align-items: center; gap: 12px; font-size: 13px; }
  .limit { display: flex; align-items: center; gap: 6px; }
  .cap { width: 52px; font: inherit; font-size: 12px; padding: 4px 6px; text-align: right;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .act { font: inherit; font-size: 11px; padding: 4px 8px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--muted); cursor: pointer; }
  .act.close { color: var(--warn-close-text); background: var(--warn-close); border-color: var(--warn-close); }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .track { height: 12px; background: color-mix(in srgb, var(--text) 6%, transparent); border-radius: 6px; overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 6px; min-width: 2px; }
  .t { color: var(--muted); font-variant-numeric: tabular-nums; }
  .del { border: none; background: transparent; color: var(--muted); cursor: pointer; font-size: 13px;
    padding: 4px 6px; border-radius: var(--radius-sm); }
  .del:hover { color: var(--accent); background: color-mix(in srgb, var(--text) 6%, transparent); }
  .barright { display: flex; align-items: center; gap: 14px; }
  .minlbl { font-size: 12px; color: var(--muted); display: flex; align-items: center; gap: 6px; }
  .minlbl select { font: inherit; font-size: 12px; padding: 4px 6px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--text); }
  .notconnected { border: 1px dashed var(--line); border-radius: var(--radius); padding: 28px; max-width: 560px; }
  .notconnected h3 { font-family: var(--font-display); margin: 0 0 10px; }
  .notconnected p { color: var(--muted); margin: 0 0 12px; }
  .notconnected ol { color: var(--text); font-size: 14px; line-height: 1.7; margin: 0 0 18px; padding-left: 20px; }
  .ncbtns { display: flex; gap: 10px; }
  .ncbtns .primary { background: var(--accent); color: var(--accent-contrast); border: none;
    padding: 9px 14px; border-radius: var(--radius-sm); cursor: pointer; font: inherit; }
  .ncbtns .ghost { background: var(--surface); color: var(--text); border: 1px solid var(--line);
    padding: 9px 14px; border-radius: var(--radius-sm); cursor: pointer; font: inherit; }
  .ncbtns .ghost:disabled { opacity: .5; cursor: default; }
</style>
