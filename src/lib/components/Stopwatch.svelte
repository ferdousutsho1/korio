<script lang="ts">
  import { nowMs, stopwatch, swStartStop, swLap, swReset, swElapsed } from "$lib/tools";
  import { formatStopwatch } from "$lib/format";
  let elapsed = $derived(swElapsed($stopwatch, $nowMs));
</script>

<div class="sw">
  <div class="time">{formatStopwatch(elapsed)}</div>
  <div class="btns">
    <button class="primary" onclick={swStartStop}>{$stopwatch.running ? "Stop" : "Start"}</button>
    <button onclick={() => swLap(elapsed)} disabled={!$stopwatch.running}>Lap</button>
    <button onclick={swReset} disabled={$stopwatch.running || (elapsed === 0 && $stopwatch.laps.length === 0)}>Reset</button>
  </div>
  {#if $stopwatch.laps.length}
    <ul class="laps">
      {#each $stopwatch.laps as lap, i}
        <li><span>Lap {$stopwatch.laps.length - i}</span><span class="t">{formatStopwatch(lap)}</span></li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .sw { display: flex; flex-direction: column; gap: 18px; align-items: center; }
  .time { font-family: var(--font-display); font-size: 56px; font-variant-numeric: tabular-nums; color: var(--text); }
  .btns { display: flex; gap: 8px; }
  .btns button { font: inherit; font-size: 14px; padding: 9px 18px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--bg); color: var(--text); cursor: pointer; }
  .btns button:disabled { opacity: .4; cursor: default; }
  .primary { background: var(--accent) !important; color: var(--accent-contrast) !important; border-color: var(--accent) !important; }
  .laps { list-style: none; margin: 0; padding: 0; width: 100%; max-width: 320px; display: flex; flex-direction: column; gap: 6px; }
  .laps li { display: flex; justify-content: space-between; font-size: 13px; color: var(--muted);
    border-bottom: 1px solid var(--line); padding-bottom: 6px; }
  .laps .t { color: var(--text); font-variant-numeric: tabular-nums; }
</style>
