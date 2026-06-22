<script lang="ts">
  import { onMount } from "svelte";
  import { siteUsageRange, clearSite, colorFor, type SiteUsage } from "$lib/api";
  import { formatDuration } from "$lib/format";

  type RangeId = "today" | "7d" | "30d";
  let range = $state<RangeId>("today");
  let sites = $state<SiteUsage[]>([]);
  let loading = $state(true);

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
    try { sites = await siteUsageRange(from, to); } catch { sites = []; }
    loading = false;
  }
  async function setRange(r: RangeId) { range = r; await load(); }
  async function remove(domain: string) {
    await clearSite(domain);
    await load();
  }

  let max = $derived(sites.reduce((m, s) => Math.max(m, s.seconds), 0));
  let total = $derived(sites.reduce((a, s) => a + s.seconds, 0));

  onMount(load);
</script>

<div class="sites">
  <div class="bar">
    <div class="ranges">
      <button class:on={range === "today"} onclick={() => setRange("today")}>Today</button>
      <button class:on={range === "7d"} onclick={() => setRange("7d")}>7 days</button>
      <button class:on={range === "30d"} onclick={() => setRange("30d")}>30 days</button>
    </div>
    <span class="total">{formatDuration(total)} total</span>
  </div>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if sites.length === 0}
    <p class="empty">No site activity tracked in this range. Enable Browser tracking in Settings and install the Korio extension.</p>
  {:else}
    <ul>
      {#each sites as s, i}
        <li>
          <span class="name" title={s.domain}>{s.domain}</span>
          <span class="track"><span class="fill" style={`width:${max > 0 ? (s.seconds / max) * 100 : 0}%;background:${colorFor(i)}`}></span></span>
          <span class="t">{formatDuration(s.seconds)}</span>
          <button class="del" aria-label={`Clear history for ${s.domain}`} onclick={() => remove(s.domain)}>✕</button>
        </li>
      {/each}
    </ul>
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
  li { display: grid; grid-template-columns: 180px 1fr auto auto; align-items: center; gap: 12px; font-size: 13px; }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .track { height: 12px; background: color-mix(in srgb, var(--text) 6%, transparent); border-radius: 6px; overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 6px; min-width: 2px; }
  .t { color: var(--muted); font-variant-numeric: tabular-nums; }
  .del { border: none; background: transparent; color: var(--muted); cursor: pointer; font-size: 13px;
    padding: 4px 6px; border-radius: var(--radius-sm); }
  .del:hover { color: var(--accent); background: color-mix(in srgb, var(--text) 6%, transparent); }
</style>
