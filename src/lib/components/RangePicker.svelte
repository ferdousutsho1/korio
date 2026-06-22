<script lang="ts">
  import { presetRange, customRange, isoDate, type Preset, type Range } from "$lib/ranges";
  let { onChange, initial = "today" }:
    { onChange: (r: Range, label: string) => void; initial?: Preset | { from: string; to: string } } = $props();

  const presets: { id: Preset; label: string }[] = [
    { id: "today", label: "Today" },
    { id: "7d", label: "Last 7 days" },
    { id: "30d", label: "Last 30 days" },
    { id: "month", label: "This month" },
  ];
  const isPreset = typeof initial === "string";
  let activeId = $state<string>(isPreset ? initial : "custom");
  let fromStr = $state(isPreset ? isoDate(presetRange(initial).from) : initial.from);
  let toStr = $state(isPreset ? isoDate(presetRange(initial).to - 86400) : initial.to);

  function choose(p: Preset, label: string) {
    activeId = p;
    const r = presetRange(p);
    fromStr = isoDate(r.from);
    toStr = isoDate(r.to - 86400);
    onChange(r, label);
  }
  function applyCustom() {
    activeId = "custom";
    onChange(customRange(fromStr, toStr), `${fromStr} → ${toStr}`);
  }
</script>

<div class="rp">
  <div class="presets">
    {#each presets as p}
      <button class:on={activeId === p.id} onclick={() => choose(p.id, p.label)}>{p.label}</button>
    {/each}
  </div>
  <div class="custom">
    <input type="date" bind:value={fromStr} aria-label="From date" max={toStr} />
    <span class="dash">→</span>
    <input type="date" bind:value={toStr} aria-label="To date" min={fromStr} />
    <button class="apply" class:on={activeId === "custom"} onclick={applyCustom}>Apply</button>
  </div>
</div>

<style>
  .rp { display: flex; flex-wrap: wrap; gap: 14px; align-items: center; justify-content: space-between; margin-bottom: 18px; }
  .presets { display: flex; gap: 6px; flex-wrap: wrap; }
  .presets button, .apply { font: inherit; font-size: 13px; padding: 7px 12px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--surface); color: var(--muted); cursor: pointer; }
  .presets button.on, .apply.on { background: var(--accent); color: var(--accent-contrast); border-color: var(--accent); }
  .presets button:hover, .apply:hover { color: var(--text); }
  .custom { display: flex; align-items: center; gap: 8px; }
  input[type="date"] { font: inherit; font-size: 13px; padding: 6px 8px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--surface); color: var(--text); color-scheme: light dark; }
  .dash { color: var(--muted); }
</style>
