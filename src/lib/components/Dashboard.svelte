<script lang="ts">
  import DonutChart from "$lib/components/DonutChart.svelte";
  import { usageToday, scoreToday, type UsageSlice } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { live, sessionTick } from "$lib/stores";

  let slices = $state<UsageSlice[]>([]);
  let score = $state(50);

  async function refresh() {
    slices = await usageToday();
    score = await scoreToday();
  }

  // runs on mount and whenever a session is saved (sessionTick bumps)
  $effect(() => {
    $sessionTick; // dependency
    refresh();
  });
</script>

<div class="grid">
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

  <div class="card score">
    <div class="label">Focus score</div>
    <div class="big">{score}</div>
    <div class="muted">productive vs. distracting</div>
  </div>

  <div class="card live">
    <div class="label">Tracking now</div>
    {#if $live.exe}
      <div class="pulse"></div>
      <div class="exe">{$live.exe}</div>
      <div class="muted">{formatDuration($live.active_seconds)} this session</div>
    {:else}
      <div class="muted">Idle — no watched app in focus.</div>
    {/if}
  </div>
</div>

<style>
  .grid { display: grid; grid-template-columns: 2fr 1fr; gap: 16px; align-items: start; }
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .chart { grid-row: span 2; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .chartrow { display: flex; gap: 22px; align-items: center; }
  .legend { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 10px; flex: 1; }
  .legend li { display: flex; align-items: center; gap: 10px; font-size: 13px; }
  .sw { width: 10px; height: 10px; border-radius: 3px; }
  .t { margin-left: auto; color: var(--muted); font-variant-numeric: tabular-nums; }
  .big { font-family: var(--font-display); font-size: 52px; color: var(--accent); line-height: 1; }
  .muted { color: var(--muted); font-size: 13px; }
  .pulse { width: 10px; height: 10px; border-radius: 50%; background: var(--accent);
    box-shadow: 0 0 0 0 var(--accent); animation: p 1.6s infinite; margin-bottom: 10px; }
  .exe { font-weight: 600; font-size: 16px; }
  @keyframes p { 0% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--accent) 60%, transparent); }
    100% { box-shadow: 0 0 0 12px transparent; } }
</style>
