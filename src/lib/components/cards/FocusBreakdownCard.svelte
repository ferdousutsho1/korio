<script lang="ts">
  import DonutChart from "$lib/components/DonutChart.svelte";
  import { usageToday, type UsageSlice } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { sessionTick } from "$lib/stores";
  let slices = $state<UsageSlice[]>([]);
  $effect(() => { $sessionTick; usageToday().then((s) => (slices = s)); });
</script>

<div class="card chart">
  <div class="label">Focus breakdown</div>
  <div class="chartrow">
    <DonutChart {slices} />
    <ul class="legend">
      {#each slices as s}
        <li><span class="sw" style={`background:${s.color}`}></span>
          <span class="n">{s.display_name}</span>
          <span class="t">{formatDuration(s.seconds)}</span></li>
      {:else}
        <li class="muted">No tracked time yet today.</li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .chartrow { display: flex; gap: 22px; align-items: center; }
  .legend { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 10px; flex: 1; }
  .legend li { display: flex; align-items: center; gap: 10px; font-size: 13px; }
  .sw { width: 10px; height: 10px; border-radius: 3px; }
  .t { margin-left: auto; color: var(--muted); font-variant-numeric: tabular-nums; }
  .muted { color: var(--muted); font-size: 13px; }
</style>
