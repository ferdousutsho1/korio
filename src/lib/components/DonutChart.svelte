<script lang="ts">
  import { formatDuration } from "$lib/format";
  type Slice = { display_name: string; color: string; seconds: number };
  let { slices = [], inner = 0.78 }: { slices?: Slice[]; inner?: number } = $props();

  const R = 70;
  const C = 2 * Math.PI * R;
  let total = $derived(slices.reduce((a, s) => a + s.seconds, 0));
  let segments = $derived.by(() => {
    let offset = 0;
    return slices.map((s) => {
      const frac = total > 0 ? s.seconds / total : 0;
      const seg = { ...s, dash: frac * C, gap: C - frac * C, offset: -offset * C, frac, start: offset };
      offset += frac;
      return seg;
    });
  });

  // Pie wedge path (used when inner === 0). Angles in turns, clockwise from 12 o'clock.
  function wedge(start: number, frac: number): string {
    if (frac <= 0) return "";
    if (frac >= 1) {
      // full circle as two arcs (a single arc can't draw 360°)
      return `M90 20 A70 70 0 1 1 89.99 20 Z`;
    }
    const a0 = start * 2 * Math.PI - Math.PI / 2;
    const a1 = (start + frac) * 2 * Math.PI - Math.PI / 2;
    const x0 = 90 + R * Math.cos(a0), y0 = 90 + R * Math.sin(a0);
    const x1 = 90 + R * Math.cos(a1), y1 = 90 + R * Math.sin(a1);
    const large = frac > 0.5 ? 1 : 0;
    return `M90 90 L${x0.toFixed(2)} ${y0.toFixed(2)} A70 70 0 ${large} 1 ${x1.toFixed(2)} ${y1.toFixed(2)} Z`;
  }
</script>

<div class="donut">
  <svg viewBox="0 0 180 180" role="img" aria-label={`Total tracked time: ${formatDuration(total)}`}>
    <title>{`Total tracked time: ${formatDuration(total)}`}</title>
    {#if inner === 0}
      {#if total === 0}
        <circle cx="90" cy="90" r={R} fill="var(--line)" />
      {/if}
      {#each segments as s}
        <path d={wedge(s.start, s.frac)} fill={s.color} />
      {/each}
    {:else}
      <g transform="translate(90,90) rotate(-90)">
        {#if total === 0}
          <circle r={R} fill="none" stroke="var(--line)" stroke-width="20" />
        {/if}
        {#each segments as s}
          <circle r={R} fill="none" stroke={s.color} stroke-width="20"
            stroke-dasharray={`${s.dash} ${s.gap}`} stroke-dashoffset={s.offset} />
        {/each}
      </g>
      <text x="90" y="84" text-anchor="middle" class="big">{formatDuration(total)}</text>
      <text x="90" y="104" text-anchor="middle" class="lbl">today</text>
    {/if}
  </svg>
</div>

<style>
  .donut { width: 180px; height: 180px; }
  .big { font-family: var(--font-display); font-size: 22px; fill: var(--text); }
  .lbl { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; fill: var(--muted); }
</style>
