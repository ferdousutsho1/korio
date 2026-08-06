<script lang="ts">
  import { eachDay, heatLevel } from "$lib/ranges";
  import type { DayTotal } from "$lib/api";
  import { formatDuration } from "$lib/format";

  let { from, to, totals = [], onPickDay, selected = null }:
    { from: number; to: number; totals?: DayTotal[]; onPickDay: (day: number) => void; selected?: number | null } = $props();

  type Cell = { day: number; secs: number; level: number };
  let byDay = $derived(new Map(totals.map((t) => [t.day, t.seconds])));
  let max = $derived(totals.reduce((m, t) => Math.max(m, t.seconds), 0));
  let days = $derived(eachDay(from, to));
  let weeks = $derived.by(() => {
    const cols: Cell[][] = [];
    let col: Cell[] = [];
    let first = true;
    for (const day of days) {
      const weekday = new Date(day * 1000).getDay(); // 0=Sun
      if (weekday === 0 && col.length) { cols.push(col); col = []; }
      // pad the first (partial) week so each day sits on its weekday row
      if (first && weekday !== 0) {
        for (let i = 0; i < weekday; i++) col.push({ day: -1, secs: 0, level: -1 });
      }
      first = false;
      const secs = byDay.get(day) ?? 0;
      col.push({ day, secs, level: heatLevel(secs, max) });
    }
    if (col.length) cols.push(col);
    return cols;
  });

  /** Month name above the first column that starts a new month (skipped when too tight). */
  let monthLabels = $derived.by(() => {
    const out: (string | null)[] = [];
    let lastMonth = -1;
    let lastLabelledAt = -99;
    for (let i = 0; i < weeks.length; i++) {
      const firstReal = weeks[i].find((c) => c.day > 0);
      if (!firstReal) { out.push(null); continue; }
      const m = new Date(firstReal.day * 1000).getMonth();
      if (m !== lastMonth && i - lastLabelledAt >= 3) {
        out.push(new Date(firstReal.day * 1000).toLocaleDateString(undefined, { month: "short" }));
        lastMonth = m; lastLabelledAt = i;
      } else {
        if (m !== lastMonth) lastMonth = m;
        out.push(null);
      }
    }
    return out;
  });

  const WEEKDAYS = ["", "Mon", "", "Wed", "", "Fri", ""];
  const fmtDay = (d: number) =>
    new Date(d * 1000).toLocaleDateString(undefined, { weekday: "short", month: "short", day: "numeric" });

  // Legend thresholds, so "more" means something concrete instead of a vague ramp.
  let steps = $derived([0.25, 0.5, 0.75, 1].map((f) => Math.round(max * f)));
  let activeDays = $derived(totals.filter((t) => t.seconds > 0).length);
  let busiest = $derived(totals.reduce<DayTotal | null>((b, t) => (!b || t.seconds > b.seconds ? t : b), null));
</script>

<div class="card">
  <div class="head">
    <div class="label">Daily activity</div>
    <div class="summary">
      {activeDays} active {activeDays === 1 ? "day" : "days"}
      {#if busiest && busiest.seconds > 0}<span class="sep">·</span>busiest {fmtDay(busiest.day)}, {formatDuration(busiest.seconds)}{/if}
    </div>
  </div>

  <div class="scroll">
    <div class="chart">
      <div class="daynames">
        <span class="months"></span>
        {#each WEEKDAYS as w}<span class="dn">{w}</span>{/each}
      </div>
      <div class="grid">
        {#each weeks as col, i}
          <div class="wk">
            <span class="months mo">{monthLabels[i] ?? ""}</span>
            {#each col as cell}
              {#if cell.level < 0}
                <span class="cell pad"></span>
              {:else}
                <button class="cell lvl{cell.level}" class:sel={cell.day === selected}
                  title={`${fmtDay(cell.day)} — ${formatDuration(cell.secs)}`}
                  onclick={() => onPickDay(cell.day)}
                  aria-label={`${fmtDay(cell.day)}, ${formatDuration(cell.secs)}`}
                  aria-pressed={cell.day === selected}></button>
              {/if}
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </div>

  <div class="legend">
    <span class="hint">Click a day to see its timeline</span>
    <span class="scale">
      <span>None</span>
      <i class="lvl0"></i>
      <i class="lvl1" title={`up to ${formatDuration(steps[0])}`}></i>
      <i class="lvl2" title={`up to ${formatDuration(steps[1])}`}></i>
      <i class="lvl3" title={`up to ${formatDuration(steps[2])}`}></i>
      <i class="lvl4" title={`up to ${formatDuration(steps[3])}`}></i>
      <span>{max > 0 ? formatDuration(max) : "More"}</span>
    </span>
  </div>
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .head { display: flex; align-items: baseline; justify-content: space-between; gap: 16px; margin-bottom: 14px; flex-wrap: wrap; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); }
  .summary { font-size: 12px; color: var(--muted); }
  .summary .sep { margin: 0 6px; opacity: .5; }

  .scroll { overflow-x: auto; padding-bottom: 6px; }
  .chart { display: flex; gap: 8px; min-width: max-content; }
  .daynames { display: flex; flex-direction: column; gap: 4px; flex-shrink: 0; }
  .dn { height: 16px; line-height: 16px; font-size: 10px; color: var(--muted); text-align: right; width: 26px; }
  .months { height: 14px; display: block; }
  .mo { font-size: 10px; color: var(--muted); white-space: nowrap; line-height: 14px; }
  .grid { display: flex; gap: 4px; }
  .wk { display: flex; flex-direction: column; gap: 4px; }
  .cell { width: 16px; height: 16px; border-radius: 3px; border: 1px solid transparent; cursor: pointer; padding: 0; }
  .cell:hover { border-color: var(--text); }
  .cell.sel { border-color: var(--accent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 35%, transparent); }
  .pad { visibility: hidden; cursor: default; }
  .lvl0 { background: color-mix(in srgb, var(--text) 7%, transparent); }
  .lvl1 { background: color-mix(in srgb, var(--accent) 30%, transparent); }
  .lvl2 { background: color-mix(in srgb, var(--accent) 55%, transparent); }
  .lvl3 { background: color-mix(in srgb, var(--accent) 78%, transparent); }
  .lvl4 { background: var(--accent); }

  .legend { display: flex; align-items: center; justify-content: space-between; gap: 12px;
    margin-top: 14px; font-size: 11px; color: var(--muted); flex-wrap: wrap; }
  .hint { font-style: italic; opacity: .8; }
  .scale { display: flex; align-items: center; gap: 4px; }
  .scale i { width: 13px; height: 13px; border-radius: 3px; display: inline-block; }
</style>
