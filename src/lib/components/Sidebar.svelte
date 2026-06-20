<script lang="ts">
  import { hiddenSections } from "$lib/sidebar";
  let { active, onNavigate }: { active: string; onNavigate: (id: string) => void } = $props();
  const items = [
    { id: "dashboard", glyph: "◷", label: "Dashboard" },
    { id: "stats", glyph: "▤", label: "Stats" },
    { id: "watchlist", glyph: "▦", label: "Watchlist" },
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
</style>
