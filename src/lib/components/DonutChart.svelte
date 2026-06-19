<script lang="ts">
  import type { UsageSlice } from "$lib/api";
  import { formatDuration } from "$lib/format";
  let { slices = [] }: { slices?: UsageSlice[] } = $props();

  const R = 70;
  const C = 2 * Math.PI * R;
  let total = $derived(slices.reduce((a, s) => a + s.seconds, 0));
  let segments = $derived.by(() => {
    let offset = 0;
    return slices.map((s) => {
      const frac = total > 0 ? s.seconds / total : 0;
      const seg = { ...s, dash: frac * C, gap: C - frac * C, offset: -offset * C };
      offset += frac;
      return seg;
    });
  });
</script>

<div class="donut">
  <svg viewBox="0 0 180 180">
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
  </svg>
</div>

<style>
  .donut { width: 180px; height: 180px; }
  .big { font-family: var(--font-display); font-size: 22px; fill: var(--text); }
  .lbl { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; fill: var(--muted); }
</style>
