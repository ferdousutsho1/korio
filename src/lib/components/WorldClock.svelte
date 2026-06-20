<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { nowMs } from "$lib/tools";

  const KEY = "korio.worldclock";
  const ZONES = [
    "UTC", "America/Los_Angeles", "America/New_York", "Europe/London", "Europe/Berlin",
    "Europe/Istanbul", "Asia/Dubai", "Asia/Karachi", "Asia/Dhaka", "Asia/Kolkata",
    "Asia/Singapore", "Asia/Tokyo", "Australia/Sydney",
  ];
  let zones = $state<string[]>([]);
  let toAdd = $state("Asia/Tokyo");

  onMount(() => {
    if (!browser) return;
    try {
      const v = JSON.parse(localStorage.getItem(KEY) || "null");
      zones = Array.isArray(v) && v.length ? v : [Intl.DateTimeFormat().resolvedOptions().timeZone, "UTC"];
    } catch { zones = [Intl.DateTimeFormat().resolvedOptions().timeZone, "UTC"]; }
  });

  function persist() { if (browser) localStorage.setItem(KEY, JSON.stringify(zones)); }
  function add() { if (!zones.includes(toAdd)) { zones = [...zones, toAdd]; persist(); } }
  function remove(z: string) { zones = zones.filter((x) => x !== z); persist(); }

  function timeIn(zone: string, _now: number) {
    try {
      return new Intl.DateTimeFormat(undefined, { timeZone: zone, hour: "2-digit", minute: "2-digit", second: "2-digit" }).format(new Date());
    } catch { return "—"; }
  }
  function dayIn(zone: string) {
    try {
      return new Intl.DateTimeFormat(undefined, { timeZone: zone, weekday: "short", month: "short", day: "numeric" }).format(new Date());
    } catch { return ""; }
  }
  const label = (z: string) => z.split("/").pop()?.replace(/_/g, " ") ?? z;
</script>

<div class="wc">
  <div class="add">
    <select bind:value={toAdd} aria-label="Time zone">
      {#each ZONES as z}<option value={z}>{label(z)} — {z}</option>{/each}
    </select>
    <button onclick={add}>Add</button>
  </div>
  <ul class="clocks">
    {#each zones as z}
      <li>
        <div class="meta"><span class="city">{label(z)}</span><span class="day">{dayIn(z)}</span></div>
        <span class="time">{timeIn(z, $nowMs)}</span>
        <button class="x" onclick={() => remove(z)} aria-label={`Remove ${label(z)}`}>×</button>
      </li>
    {:else}
      <li class="muted">No clocks. Add a time zone above.</li>
    {/each}
  </ul>
</div>

<style>
  .wc { display: flex; flex-direction: column; gap: 16px; }
  .add { display: flex; gap: 8px; }
  select { flex: 1; font: inherit; font-size: 13px; padding: 8px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .add button { font: inherit; padding: 8px 16px; border-radius: var(--radius-sm); border: none;
    background: var(--accent); color: var(--accent-contrast); cursor: pointer; }
  .clocks { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px; }
  .clocks li { display: flex; align-items: center; gap: 14px; border-bottom: 1px solid var(--line); padding: 10px 0; }
  .meta { display: flex; flex-direction: column; }
  .city { font-weight: 600; }
  .day { font-size: 11px; color: var(--muted); }
  .time { margin-left: auto; font-family: var(--font-display); font-size: 24px; font-variant-numeric: tabular-nums; }
  .x { border: none; background: none; color: var(--muted); font-size: 20px; cursor: pointer; }
  .muted { color: var(--muted); font-size: 13px; }
</style>
