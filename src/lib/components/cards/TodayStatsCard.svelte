<script lang="ts">
  import { usageToday, type UsageSlice } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { sessionTick } from "$lib/stores";
  let slices = $state<UsageSlice[]>([]);
  let total = $derived(slices.reduce((a, s) => a + s.seconds, 0));
  $effect(() => { $sessionTick; usageToday().then((s) => (slices = s)); });
</script>

<div class="card">
  <div class="label">Today's stats</div>
  <div class="big">{formatDuration(total)}</div>
  <div class="muted">{slices.length} app{slices.length === 1 ? "" : "s"} tracked today</div>
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .big { font-family: var(--font-display); font-size: 36px; color: var(--text); line-height: 1; }
  .muted { color: var(--muted); font-size: 13px; margin-top: 6px; }
</style>
