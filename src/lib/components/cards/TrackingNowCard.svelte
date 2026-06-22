<script lang="ts">
  import { formatDuration } from "$lib/format";
  import { live } from "$lib/stores";
</script>

<div class="card">
  <div class="label">Tracking now</div>
  {#if $live.exe}
    <div class="pulse"></div>
    <div class="exe">{$live.exe}</div>
    <div class="muted">{formatDuration($live.active_seconds)} this session</div>
  {:else}
    <div class="muted">Idle — no watched app in focus.</div>
  {/if}
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .muted { color: var(--muted); font-size: 13px; }
  .pulse { width: 10px; height: 10px; border-radius: 50%; background: var(--accent);
    box-shadow: 0 0 0 0 var(--accent); animation: p 1.6s infinite; margin-bottom: 10px; }
  .exe { font-weight: 600; font-size: 16px; }
  @keyframes p { 0% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--accent) 60%, transparent); }
    100% { box-shadow: 0 0 0 12px transparent; } }
</style>
