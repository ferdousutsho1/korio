<script lang="ts">
  import { goalsProgress } from "$lib/api";
  import type { GoalProgress } from "$lib/goals";
  import { sessionTick } from "$lib/stores";
  let progress = $state<GoalProgress[]>([]);
  $effect(() => { $sessionTick; goalsProgress().then((p) => (progress = p)); });
  let met = $derived(progress.filter((p) => p.met_today).length);
</script>

<div class="card">
  <div class="label">Goals</div>
  {#if progress.length === 0}
    <div class="muted">No goals set.</div>
  {:else}
    <div class="big">{met}/{progress.length}</div>
    <div class="muted">goals met today</div>
  {/if}
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .big { font-family: var(--font-display); font-size: 36px; color: var(--accent); line-height: 1; }
  .muted { color: var(--muted); font-size: 13px; margin-top: 6px; }
</style>
