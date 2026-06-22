<script lang="ts">
  import { scoreToday } from "$lib/api";
  import { sessionTick } from "$lib/stores";
  let score = $state(50);
  $effect(() => { $sessionTick; scoreToday().then((s) => (score = s)); });
</script>

<div class="card">
  <div class="label">Focus score</div>
  <div class="big">{score}</div>
  <div class="muted">productive vs. distracting</div>
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .big { font-family: var(--font-display); font-size: 52px; color: var(--accent); line-height: 1; }
  .muted { color: var(--muted); font-size: 13px; }
</style>
