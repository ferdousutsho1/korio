<script lang="ts">
  import type { UsageSlice } from "$lib/api";
  import { formatDuration } from "$lib/format";
  let { slices = [] }: { slices?: UsageSlice[] } = $props();
  let max = $derived(slices.reduce((m, s) => Math.max(m, s.seconds), 0));
  let total = $derived(slices.reduce((a, s) => a + s.seconds, 0));
</script>

<div class="card">
  <div class="label">Usage breakdown <span class="tot">{formatDuration(total)} total</span></div>
  {#if slices.length === 0}
    <p class="empty">No tracked time in this range.</p>
  {:else}
    <ul>
      {#each slices as s}
        <li>
          <span class="name">{s.display_name}</span>
          <span class="track"><span class="fill" style={`width:${max > 0 ? (s.seconds / max) * 100 : 0}%;background:${s.color}`}></span></span>
          <span class="t">{formatDuration(s.seconds)}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted);
    margin-bottom: 14px; display: flex; justify-content: space-between; }
  .tot { letter-spacing: 0; text-transform: none; font-size: 12px; }
  .empty { color: var(--muted); font-size: 13px; margin: 6px 0; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
  li { display: grid; grid-template-columns: 140px 1fr auto; align-items: center; gap: 12px; font-size: 13px; }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .track { height: 12px; background: color-mix(in srgb, var(--text) 6%, transparent); border-radius: 6px; overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 6px; min-width: 2px; }
  .t { color: var(--muted); font-variant-numeric: tabular-nums; }
</style>
