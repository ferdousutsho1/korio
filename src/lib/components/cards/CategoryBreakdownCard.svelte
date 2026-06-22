<script lang="ts">
  import DonutChart from "$lib/components/DonutChart.svelte";
  import { usageByCategory, type CategoryUsage } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { dayStartLocal } from "$lib/ranges";
  import { sessionTick } from "$lib/stores";

  let rows = $state<CategoryUsage[]>([]);
  $effect(() => {
    $sessionTick;
    const day = dayStartLocal(new Date());
    usageByCategory(day, day + 86400).then((r) => (rows = r));
  });
  // DonutChart reads display_name/color/seconds:
  let slices = $derived(rows.map((r) => ({ display_name: r.name, color: r.color, seconds: r.seconds })));
</script>

<div class="card">
  <div class="label">Category breakdown</div>
  <div class="chartrow">
    <DonutChart {slices} inner={0} />
    <ul class="legend">
      {#each rows as r}
        <li><span class="sw" style={`background:${r.color}`}></span>
          <span class="n">{r.name}</span>
          <span class="t">{formatDuration(r.seconds)}</span></li>
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
