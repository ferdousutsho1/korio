<script lang="ts">
  import type { SessionRow } from "$lib/api";
  import { formatDuration } from "$lib/format";
  let { dayStart, sessions = [] }: { dayStart: number; sessions?: SessionRow[] } = $props();
  const DAY = 86400;

  let blocks = $derived(sessions.map((s, i) => {
    const start = Math.max(0, s.started_at - dayStart);
    const end = Math.min(DAY, s.ended_at - dayStart);
    const clock = new Date(s.started_at * 1000).toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
    return {
      i,
      left: (start / DAY) * 100,
      // A one-minute session is ~0.07% wide — floor it so every session stays clickable.
      width: Math.max(0.6, ((end - start) / DAY) * 100),
      color: s.color,
      name: s.display_name,
      clock,
      secs: s.active_seconds,
      label: `${s.display_name} · ${clock} · ${formatDuration(s.active_seconds)}`,
    };
  }));

  let hovered = $state<number | null>(null);
  let active = $derived(hovered === null ? null : blocks[hovered] ?? null);

  /** Per-app totals for the selected day, biggest first — the ribbon's key. */
  let legend = $derived.by(() => {
    const map = new Map<string, { name: string; color: string; secs: number }>();
    for (const s of sessions) {
      const cur = map.get(s.display_name);
      if (cur) cur.secs += s.active_seconds;
      else map.set(s.display_name, { name: s.display_name, color: s.color, secs: s.active_seconds });
    }
    return [...map.values()].sort((a, b) => b.secs - a.secs);
  });
  let total = $derived(sessions.reduce((a, s) => a + s.active_seconds, 0));

  let label = $derived(
    new Date(dayStart * 1000).toLocaleDateString(undefined, { weekday: "long", month: "short", day: "numeric" })
  );
  // A tick every 2h; only every other one gets a printed label so they never collide.
  const ticks = Array.from({ length: 13 }, (_, i) => i * 2);
</script>

<div class="card">
  <div class="head">
    <div class="label">Timeline</div>
    <div class="day">{label}<span class="sep">·</span>{formatDuration(total)} tracked</div>
  </div>

  <div class="plot">
    <div class="gridlines" aria-hidden="true">
      {#each ticks as h}<span class="gl" class:major={h % 6 === 0} style={`left:${(h / 24) * 100}%`}></span>{/each}
    </div>
    <div class="ribbon">
      {#each blocks as b (b.i)}
        <button class="blk" style={`left:${b.left}%;width:${b.width}%;background:${b.color}`}
          title={b.label} aria-label={b.label}
          onmouseenter={() => (hovered = b.i)} onmouseleave={() => (hovered = null)}
          onfocus={() => (hovered = b.i)} onblur={() => (hovered = null)}></button>
      {/each}
      {#if blocks.length === 0}<span class="empty">No sessions this day.</span>{/if}
    </div>
    <div class="axis" aria-hidden="true">
      {#each ticks as h}
        <span class="tk" style={`left:${(h / 24) * 100}%`}>{h % 4 === 0 ? `${String(h).padStart(2, "0")}:00` : ""}</span>
      {/each}
    </div>
  </div>

  <div class="readout" aria-live="polite">
    {#if active}
      <span class="sw" style={`background:${active.color}`}></span>
      <strong>{active.name}</strong><span class="sep">·</span>{active.clock}<span class="sep">·</span>{formatDuration(active.secs)}
    {:else if blocks.length}
      <span class="hint">Hover a block for details</span>
    {/if}
  </div>

  {#if legend.length}
    <ul class="legend">
      {#each legend as l (l.name)}
        <li><span class="sw" style={`background:${l.color}`}></span>
          <span class="n">{l.name}</span><span class="t">{formatDuration(l.secs)}</span></li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .head { display: flex; align-items: baseline; justify-content: space-between; gap: 16px;
    margin-bottom: 16px; flex-wrap: wrap; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); }
  .day { font-size: 12px; color: var(--muted); }
  .sep { margin: 0 6px; opacity: .5; }

  .plot { position: relative; }
  .ribbon { position: relative; height: 46px; background: color-mix(in srgb, var(--text) 5%, transparent);
    border: 1px solid var(--line); border-radius: 6px; overflow: hidden; }
  .gridlines { position: absolute; inset: 0 0 22px 0; pointer-events: none; z-index: 1; }
  .gl { position: absolute; top: 0; bottom: 0; width: 1px;
    background: color-mix(in srgb, var(--text) 8%, transparent); }
  .gl.major { background: color-mix(in srgb, var(--text) 18%, transparent); }
  .blk { position: absolute; top: 5px; bottom: 5px; border: none; padding: 0; border-radius: 3px;
    min-width: 3px; cursor: pointer; z-index: 2; }
  .blk:hover, .blk:focus-visible { outline: 2px solid var(--text); outline-offset: 1px; z-index: 3; }
  .empty { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
    color: var(--muted); font-size: 12px; }
  .axis { position: relative; height: 18px; margin-top: 4px; }
  .tk { position: absolute; top: 0; transform: translateX(-50%); font-size: 10px;
    color: var(--muted); font-variant-numeric: tabular-nums; white-space: nowrap; }

  .readout { min-height: 20px; margin-top: 8px; font-size: 12px; color: var(--muted);
    display: flex; align-items: center; gap: 6px; }
  .readout strong { color: var(--text); font-weight: 600; }
  .hint { font-style: italic; opacity: .8; }

  .legend { list-style: none; margin: 14px 0 0; padding: 12px 0 0; border-top: 1px solid var(--line);
    display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px 18px; }
  .legend li { display: flex; align-items: center; gap: 8px; font-size: 12px; min-width: 0; }
  .legend .n { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .legend .t { margin-left: auto; color: var(--muted); font-variant-numeric: tabular-nums; }
  .sw { width: 10px; height: 10px; border-radius: 3px; flex-shrink: 0; }
</style>
