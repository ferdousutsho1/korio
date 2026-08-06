<script lang="ts">
  import DonutChart from "$lib/components/DonutChart.svelte";
  import { siteSlices, type SiteSlice } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { sessionTick } from "$lib/stores";

  let slices = $state<SiteSlice[]>([]);
  function dayBounds(): [number, number] {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    const from = Math.floor(d.getTime() / 1000);
    return [from, from + 86400];
  }
  // Site time is written by the tracker on the same cadence as app sessions.
  $effect(() => {
    $sessionTick;
    const [from, to] = dayBounds();
    siteSlices(from, to).then((s) => (slices = s)).catch(() => {});
  });
  let total = $derived(slices.reduce((a, s) => a + s.seconds, 0));
</script>

<div class="card chart">
  <div class="label">Site focus breakdown</div>
  <div class="chartrow">
    <DonutChart slices={slices.map((s) => ({
      display_name: s.display_name || s.domain, color: s.color, seconds: s.seconds,
    }))} />
    <ul class="legend">
      {#each slices.slice(0, 8) as s (s.domain)}
        <li><span class="sw" style={`background:${s.color}`}></span>
          <span class="n" title={s.domain}>{s.display_name || s.domain}</span>
          <span class="t">{formatDuration(s.seconds)}</span></li>
      {:else}
        <li class="muted">No site time yet today.</li>
      {/each}
      {#if slices.length > 8}
        <li class="muted">+{slices.length - 8} more · {formatDuration(total)} total</li>
      {/if}
    </ul>
  </div>
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .chartrow { display: flex; gap: 22px; align-items: center; }
  .legend { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 10px; flex: 1; min-width: 0; }
  .legend li { display: flex; align-items: center; gap: 10px; font-size: 13px; min-width: 0; }
  .sw { width: 10px; height: 10px; border-radius: 3px; flex-shrink: 0; }
  .n { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .t { margin-left: auto; color: var(--muted); font-variant-numeric: tabular-nums; }
  .muted { color: var(--muted); font-size: 13px; }
</style>
