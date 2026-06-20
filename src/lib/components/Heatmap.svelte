<script lang="ts">
  import { eachDay, heatLevel } from "$lib/ranges";
  import type { DayTotal } from "$lib/api";
  import { formatDuration } from "$lib/format";

  let { from, to, totals = [], onPickDay }:
    { from: number; to: number; totals?: DayTotal[]; onPickDay: (day: number) => void } = $props();

  let byDay = $derived(new Map(totals.map((t) => [t.day, t.seconds])));
  let max = $derived(totals.reduce((m, t) => Math.max(m, t.seconds), 0));
  let days = $derived(eachDay(from, to));
  let weeks = $derived.by(() => {
    const cols: { day: number; secs: number; level: number }[][] = [];
    let col: { day: number; secs: number; level: number }[] = [];
    for (const day of days) {
      const weekday = new Date(day * 1000).getDay(); // 0=Sun
      if (weekday === 0 && col.length) { cols.push(col); col = []; }
      const secs = byDay.get(day) ?? 0;
      col.push({ day, secs, level: heatLevel(secs, max) });
    }
    if (col.length) cols.push(col);
    return cols;
  });
  const fmtDay = (d: number) => new Date(d * 1000).toLocaleDateString(undefined, { month: "short", day: "numeric" });
</script>

<div class="card">
  <div class="label">Daily activity</div>
  <div class="grid">
    {#each weeks as col}
      <div class="wk">
        {#each col as cell}
          <button class="cell lvl{cell.level}" title={`${fmtDay(cell.day)} — ${formatDuration(cell.secs)}`}
            onclick={() => onPickDay(cell.day)} aria-label={`${fmtDay(cell.day)}, ${formatDuration(cell.secs)}`}></button>
        {/each}
      </div>
    {/each}
  </div>
  <div class="legend"><span>Less</span><i class="lvl0"></i><i class="lvl1"></i><i class="lvl2"></i><i class="lvl3"></i><i class="lvl4"></i><span>More</span></div>
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .grid { display: flex; gap: 4px; overflow-x: auto; padding-bottom: 4px; }
  .wk { display: flex; flex-direction: column; gap: 4px; }
  .cell { width: 14px; height: 14px; border-radius: 3px; border: none; cursor: pointer; padding: 0; }
  .cell:hover { outline: 1px solid var(--accent); }
  .lvl0 { background: color-mix(in srgb, var(--text) 7%, transparent); }
  .lvl1 { background: color-mix(in srgb, var(--accent) 30%, transparent); }
  .lvl2 { background: color-mix(in srgb, var(--accent) 55%, transparent); }
  .lvl3 { background: color-mix(in srgb, var(--accent) 78%, transparent); }
  .lvl4 { background: var(--accent); }
  .legend { display: flex; align-items: center; gap: 4px; margin-top: 12px; font-size: 11px; color: var(--muted); }
  .legend i { width: 12px; height: 12px; border-radius: 3px; display: inline-block; }
</style>
