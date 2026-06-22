<script lang="ts">
  import { CARD_CATALOG, type CardState } from "$lib/dashboard";
  let { layout, onChange, onClose }:
    { layout: CardState[]; onChange: (l: CardState[]) => void; onClose: () => void } = $props();

  const title = (id: string) => CARD_CATALOG.find((c) => c.id === id)?.title ?? id;
  let dragIndex = $state<number | null>(null);

  function toggle(i: number) {
    const next = layout.map((c, j) => (j === i ? { ...c, enabled: !c.enabled } : c));
    onChange(next);
  }
  function onDrop(i: number) {
    if (dragIndex === null || dragIndex === i) return;
    const next = [...layout];
    const [moved] = next.splice(dragIndex, 1);
    next.splice(i, 0, moved);
    dragIndex = null;
    onChange(next);
  }
</script>

<div class="overlay" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
  <div class="sheet">
    <h3>Customize dashboard</h3>
    <ul>
      {#each layout as c, i (c.id)}
        <li draggable="true"
          ondragstart={() => (dragIndex = i)}
          ondragover={(e) => e.preventDefault()}
          ondrop={() => onDrop(i)}>
          <span class="grip" aria-hidden="true">⠿</span>
          <span class="name">{title(c.id)}</span>
          <button class="vis" class:on={c.enabled} role="switch" aria-checked={c.enabled}
            aria-label={`Toggle ${title(c.id)}`} onclick={() => toggle(i)}>
            {c.enabled ? "Shown" : "Hidden"}
          </button>
        </li>
      {/each}
    </ul>
    <button class="done" onclick={onClose}>Done</button>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,.35); display: flex; align-items: center; justify-content: center; z-index: 50; }
  .sheet { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); width: 380px; max-height: 70vh; overflow: auto; padding: 20px; }
  h3 { font-family: var(--font-display); margin: 0 0 14px; }
  ul { list-style: none; padding: 0; margin: 0 0 14px; display: flex; flex-direction: column; gap: 6px; }
  li { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); cursor: grab; }
  .grip { color: var(--muted); }
  .name { flex: 1; font-size: 13px; }
  .vis { font: inherit; font-size: 12px; padding: 4px 10px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--surface); color: var(--muted); cursor: pointer; }
  .vis.on { border-color: var(--accent); color: var(--accent); }
  .done { font: inherit; font-size: 13px; padding: 8px 16px; border-radius: var(--radius-sm);
    border: none; background: var(--accent); color: var(--accent-contrast); cursor: pointer; }
</style>
