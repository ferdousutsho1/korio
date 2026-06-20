<script lang="ts">
  import type { SessionRow } from "$lib/api";
  import { formatDuration } from "$lib/format";
  let { dayStart, sessions = [] }: { dayStart: number; sessions?: SessionRow[] } = $props();
  const DAY = 86400;
  let blocks = $derived(sessions.map((s) => {
    const start = Math.max(0, s.started_at - dayStart);
    const end = Math.min(DAY, s.ended_at - dayStart);
    return {
      left: (start / DAY) * 100,
      width: Math.max(0.4, ((end - start) / DAY) * 100),
      color: s.color,
      label: `${s.display_name} · ${formatDuration(s.active_seconds)}`,
    };
  }));
  let label = $derived(
    new Date(dayStart * 1000).toLocaleDateString(undefined, { weekday: "short", month: "short", day: "numeric" })
  );
  const hours = [0, 6, 12, 18, 24];
</script>

<div class="card">
  <div class="label">Timeline <span class="day">{label}</span></div>
  <div class="ribbon">
    {#each blocks as b}
      <div class="blk" style={`left:${b.left}%;width:${b.width}%;background:${b.color}`} title={b.label}></div>
    {/each}
    {#if blocks.length === 0}<span class="empty">No sessions this day.</span>{/if}
  </div>
  <div class="axis">{#each hours as h}<span>{h}:00</span>{/each}</div>
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted);
    margin-bottom: 14px; display: flex; justify-content: space-between; }
  .day { letter-spacing: 0; text-transform: none; font-size: 12px; }
  .ribbon { position: relative; height: 34px; background: color-mix(in srgb, var(--text) 6%, transparent);
    border-radius: 6px; overflow: hidden; }
  .blk { position: absolute; top: 4px; bottom: 4px; border-radius: 3px; min-width: 2px; }
  .empty { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
    color: var(--muted); font-size: 12px; }
  .axis { display: flex; justify-content: space-between; margin-top: 6px; font-size: 10px; color: var(--muted); }
</style>
