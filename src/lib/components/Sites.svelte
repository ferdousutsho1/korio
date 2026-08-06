<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { siteUsageRange, siteUsageToday, clearSite, colorFor, listSiteCaps, setSiteLimit,
    setSiteName, setSiteCategory, listCategories, browserStatus,
    type SiteUsage, type SiteCap, type BrowserStatus, type Category } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { readJSON, writeJSON } from "$lib/prefs";
  import { navIntent } from "$lib/nav";

  type RangeId = "today" | "7d" | "30d";
  let range = $state<RangeId>("today");
  type Row = {
    domain: string; name: string; seconds: number; todaySeconds: number;
    cap: number; action: string; categoryId: number | null;
  };
  let rows = $state<Row[]>([]);
  let loading = $state(true);
  let categories = $state<Category[]>([]);
  let status = $state<BrowserStatus | null>(null);
  let checking = $state(false);
  let connected = $derived(status?.connected === true);
  let dismissedBanner = $state(false);
  let statusPoll: ReturnType<typeof setInterval> | null = null;
  let usagePoll: ReturnType<typeof setInterval> | null = null;

  async function refreshStatus() {
    checking = true;
    try { status = await browserStatus(); } catch { /* keep last status on transient failure */ }
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
    const [from, to] = bounds(range);
    let usage: SiteUsage[] = [];
    let today: SiteUsage[] = [];
    let caps: SiteCap[] = [];
    try {
      [usage, today, caps] = await Promise.all([
        siteUsageRange(from, to),
        // Meters always measure the DAILY limit, whatever range is on screen.
        range === "today" ? siteUsageRange(from, to) : siteUsageToday(),
        listSiteCaps(),
      ]);
    } catch { usage = []; today = []; caps = []; }
    const capMap = new Map(caps.map((c) => [c.domain, c]));
    const todayMap = new Map(today.map((t) => [t.domain, t.seconds]));
    const seen = new Set<string>();
    const merged: Row[] = usage.map((u) => {
      seen.add(u.domain);
      const c = capMap.get(u.domain);
      return {
        domain: u.domain, name: c?.display_name || u.domain, seconds: u.seconds,
        todaySeconds: todayMap.get(u.domain) ?? 0,
        cap: c?.daily_cap_seconds ?? 0, action: c?.limit_action ?? "warn",
        categoryId: c?.category_id ?? null,
      };
    });
    for (const c of caps) {
      if (!seen.has(c.domain) && c.daily_cap_seconds > 0) {
        merged.push({
          domain: c.domain, name: c.display_name || c.domain, seconds: 0,
          todaySeconds: todayMap.get(c.domain) ?? 0,
          cap: c.daily_cap_seconds, action: c.limit_action, categoryId: c.category_id,
        });
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
  async function commitName(row: Row, value: string) {
    const next = value.trim();
    if (next === row.name) return;
    // Typing the domain back in (or clearing the box) drops the override.
    await setSiteName(row.domain, next && next !== row.domain ? next : null);
    await load();
  }
  async function commitCategory(row: Row, value: string) {
    await setSiteCategory(row.domain, value === "" ? null : Number(value));
    await load();
  }

  let minSeconds = $state<number>(readJSON<number>("korio.sites.minSeconds", 0));
  function setMin(v: number) { minSeconds = v; writeJSON("korio.sites.minSeconds", v); }
  // Visible rows: at/above threshold, OR has a configured cap (never hide a limit you set).
  let visible = $derived(rows.filter((r) => r.seconds >= minSeconds || r.cap > 0));
  let visibleTotal = $derived(visible.reduce((a, r) => a + r.seconds, 0));
  let max = $derived(rows.reduce((m, s) => Math.max(m, s.seconds), 0));

  onMount(async () => {
    try { categories = await listCategories(); } catch { categories = []; }
    await refreshStatus();
    await load();
    statusPoll = setInterval(refreshStatus, 15_000);
    // Site stats keep working with the browser closed — they just stop growing.
    usagePoll = setInterval(load, 60_000);
  });
  onDestroy(() => {
    if (statusPoll) clearInterval(statusPoll);
    if (usagePoll) clearInterval(usagePoll);
  });
</script>

<div class="sites">
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
        {@const over = s.cap > 0 && s.todaySeconds > s.cap}
        <li>
          <span class="who">
            <input class="name" value={s.name} title={s.domain} aria-label={`Name for ${s.domain}`}
              onchange={(e) => commitName(s, e.currentTarget.value)} />
            {#if s.name !== s.domain}<span class="dom">{s.domain}</span>{/if}
            <span class="track"><span class="fill"
              style={`width:${max > 0 ? (s.seconds / max) * 100 : 0}%;background:${colorFor(i)}`}></span></span>
            {#if s.cap > 0}
              <span class="meter"><span class="mfill" class:over
                style={`width:${Math.min(s.todaySeconds / s.cap, 1) * 100}%;${over ? "" : `background:${colorFor(i)}`}`}></span></span>
              {#if over}
                <span class="overtxt">Over limit — {formatDuration(s.todaySeconds - s.cap)} past {formatDuration(s.cap)}</span>
              {:else}
                <span class="captxt">{formatDuration(s.todaySeconds)} of {formatDuration(s.cap)} daily limit</span>
              {/if}
            {/if}
          </span>
          <span class="t">{formatDuration(s.seconds)}</span>
          <select class="cat" aria-label={`Category for ${s.name}`}
            value={s.categoryId == null ? "" : String(s.categoryId)}
            onchange={(e) => commitCategory(s, e.currentTarget.value)}>
            <option value="">Uncategorized</option>
            {#each categories as c (c.id)}<option value={String(c.id)}>{c.name}</option>{/each}
          </select>
          <span class="limit">
            <input class="cap" type="number" min="0" step="5" value={Math.round(s.cap / 60)}
              title="Daily limit (minutes, 0 = off)" aria-label={`Daily limit minutes for ${s.name}`}
              onchange={(e) => commitCap(s, +e.currentTarget.value, s.action)} />
            <span class="unit">min</span>
            <button class="act" class:close={s.action === "close"}
              title={s.action === "close" ? "Auto-close tab when over limit" : "Warn when over limit"}
              onclick={() => commitCap(s, Math.round(s.cap / 60), s.action === "close" ? "warn" : "close")}>
              {s.action === "close" ? "Auto-close" : "Warn"}
            </button>
          </span>
          <button class="del" aria-label={`Clear history for ${s.name}`} onclick={() => remove(s.domain)}>✕</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if !connected && !dismissedBanner}
  <div class="banner" role="status">
    <span class="dot"></span>
    <span class="msg">Browser not connected — live site tracking is paused. Your stats above still show.</span>
    <button class="link" onclick={goSettings}>Settings</button>
    <button class="link" onclick={recheck} disabled={checking}>{checking ? "Checking…" : "Retry"}</button>
    <button class="link x" onclick={() => (dismissedBanner = true)} aria-label="Dismiss">×</button>
  </div>
{/if}

<style>
  .sites { max-width: 820px; padding-bottom: 48px; }
  .bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 18px; }
  .ranges { display: flex; gap: 6px; }
  .ranges button { padding: 6px 12px; border: 1px solid var(--line); background: var(--surface);
    color: var(--muted); border-radius: var(--radius-sm); font: inherit; font-size: 13px; cursor: pointer; }
  .ranges button.on { color: var(--accent-contrast); background: var(--accent); border-color: var(--accent); }
  .total { font-size: 13px; color: var(--muted); font-variant-numeric: tabular-nums; }
  .empty { color: var(--muted); font-size: 13px; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
  li { display: grid; grid-template-columns: 1fr auto auto auto auto; align-items: center; gap: 12px; font-size: 13px; }
  .who { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
  .name { font: inherit; font-size: 13px; font-weight: 600; width: 100%; padding: 2px 4px;
    border: 1px solid transparent; border-radius: var(--radius-sm); background: transparent;
    color: var(--text); overflow: hidden; text-overflow: ellipsis; }
  .name:hover, .name:focus { border-color: var(--line); background: var(--bg); outline: none; }
  .dom { font-size: 11px; color: var(--muted); padding-left: 4px; }
  .limit { display: flex; align-items: center; gap: 6px; }
  .cap { width: 52px; font: inherit; font-size: 12px; padding: 4px 6px; text-align: right;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .unit { color: var(--muted); font-size: 11px; }
  .cat { font: inherit; font-size: 12px; max-width: 130px; padding: 4px 6px;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .act { font: inherit; font-size: 11px; padding: 4px 8px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--muted); cursor: pointer; }
  .act.close { color: var(--warn-close-text); background: var(--warn-close); border-color: var(--warn-close); }
  .track { height: 12px; margin-top: 2px; background: color-mix(in srgb, var(--text) 6%, transparent);
    border-radius: 6px; overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 6px; min-width: 2px; }
  .meter { height: 5px; width: 100%; margin-top: 4px; border-radius: 3px;
    background: color-mix(in srgb, var(--text) 8%, transparent); overflow: hidden; }
  .mfill { display: block; height: 100%; border-radius: 3px; min-width: 2px; }
  .mfill.over { background: var(--warn-close); }
  .overtxt { color: var(--warn-close); font-size: 11px; padding-left: 4px; }
  .captxt { color: var(--muted); font-size: 11px; padding-left: 4px; }
  .t { color: var(--muted); font-variant-numeric: tabular-nums; align-self: center; }
  .del { border: none; background: transparent; color: var(--muted); cursor: pointer; font-size: 13px;
    padding: 4px 6px; border-radius: var(--radius-sm); }
  .del:hover { color: var(--accent); background: color-mix(in srgb, var(--text) 6%, transparent); }
  .barright { display: flex; align-items: center; gap: 14px; }
  .minlbl { font-size: 12px; color: var(--muted); display: flex; align-items: center; gap: 6px; }
  .minlbl select { font: inherit; font-size: 12px; padding: 4px 6px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--text); }

  .banner { position: fixed; left: 50%; bottom: 16px; transform: translateX(-50%); z-index: 20;
    display: flex; align-items: center; gap: 10px; max-width: 90vw;
    background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius-sm);
    padding: 9px 14px; font-size: 12px; color: var(--muted); box-shadow: 0 6px 24px rgba(0,0,0,.18); }
  .banner .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--warn-close); flex-shrink: 0; }
  .banner .msg { min-width: 0; }
  .link { border: none; background: transparent; color: var(--accent); cursor: pointer;
    font: inherit; font-size: 12px; padding: 2px 4px; text-decoration: underline; }
  .link:disabled { opacity: .5; cursor: default; text-decoration: none; }
  .link.x { color: var(--muted); text-decoration: none; font-size: 16px; line-height: 1; }
</style>
