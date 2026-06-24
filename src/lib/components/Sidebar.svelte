<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { hiddenSections } from "$lib/sidebar";
  import { theme, toggleTheme } from "$lib/theme";
  let { active, onNavigate }: { active: string; onNavigate: (id: string) => void } = $props();
  let version = $state("");
  onMount(() => { getVersion().then((v) => (version = v)).catch(() => {}); });
  const items = [
    { id: "dashboard", glyph: "◷", label: "Dashboard" },
    { id: "stats", glyph: "▤", label: "Stats" },
    { id: "watchlist", glyph: "▦", label: "Watchlist" },
    { id: "sites", glyph: "🌐", label: "Sites" },
    { id: "tools", glyph: "⏱", label: "Tools" },
    { id: "tasks", glyph: "✓", label: "Tasks" },
    { id: "notes", glyph: "🗒", label: "Notes" },
    { id: "goals", glyph: "🎯", label: "Goals" },
    { id: "settings", glyph: "⚙", label: "Settings" },
  ];
  let visible = $derived(items.filter((it) => !$hiddenSections.includes(it.id)));
</script>

<nav class="side">
  <div class="brand">Korio</div>
  {#each visible as it}
    <button class:on={active === it.id} onclick={() => onNavigate(it.id)} title={it.label}>
      <span class="g">{it.glyph}</span><span class="l">{it.label}</span>
    </button>
  {/each}
  <div class="foot">
    <button class="theme" aria-label={$theme === "light" ? "Switch to dark theme" : "Switch to light theme"}
      onclick={toggleTheme}>{$theme === "light" ? "☾" : "☀"}</button>
    <span class="ver" title="Korio version">{version ? `Korio v${version}` : ""}</span>
  </div>
</nav>

<style>
  .side { width: 200px; background: var(--sidebar); border-right: 1px solid var(--line);
    display: flex; flex-direction: column; gap: 4px; padding: 18px 12px; }
  .brand { font-family: var(--font-display); font-size: 22px; letter-spacing: -.5px;
    color: var(--text); padding: 4px 8px 16px; }
  button { display: flex; align-items: center; gap: 10px; padding: 9px 10px; border: none;
    background: transparent; color: var(--muted); border-radius: var(--radius-sm);
    font: inherit; font-size: 14px; cursor: pointer; text-align: left; }
  button:hover { color: var(--text); background: color-mix(in srgb, var(--text) 6%, transparent); }
  button.on { color: var(--accent-contrast); background: var(--accent); }
  .g { width: 18px; text-align: center; }
  .foot { margin-top: auto; display: flex; align-items: center; gap: 8px; padding: 12px 8px 2px; }
  .theme { width: 30px; height: 30px; flex-shrink: 0; padding: 0; justify-content: center; line-height: 1;
    border-radius: var(--radius-sm); border: 1px solid var(--line); background: var(--surface);
    color: var(--text); cursor: pointer; font-size: 14px; }
  .ver { font-size: 11px; color: var(--muted); letter-spacing: .3px; font-variant-numeric: tabular-nums; }
</style>
