<script lang="ts">
  import Stopwatch from "$lib/components/Stopwatch.svelte";
  import CountdownTimer from "$lib/components/CountdownTimer.svelte";
  import WorldClock from "$lib/components/WorldClock.svelte";
  const tabs = [
    { id: "stopwatch", label: "Stopwatch" },
    { id: "timer", label: "Timer" },
    { id: "clock", label: "World Clock" },
  ];
  let tab = $state("stopwatch");
</script>

<div class="tools">
  <div class="tabs">
    {#each tabs as t}
      <button class:on={tab === t.id} onclick={() => (tab = t.id)}>{t.label}</button>
    {/each}
  </div>
  <div class="panel">
    {#if tab === "stopwatch"}<Stopwatch />{:else if tab === "timer"}<CountdownTimer />{:else}<WorldClock />{/if}
  </div>
</div>

<style>
  .tools { display: flex; flex-direction: column; gap: 20px; max-width: 640px; }
  .tabs { display: flex; gap: 4px; }
  .tabs button { font: inherit; font-size: 13px; padding: 8px 14px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--muted); cursor: pointer; }
  .tabs button.on { background: var(--accent); color: var(--accent-contrast); border-color: var(--accent); }
  .panel { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 28px; }
</style>
