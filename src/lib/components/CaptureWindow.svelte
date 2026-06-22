<script lang="ts">
  import { onMount } from "svelte";
  import { addTask, addNote, updateNote, hideCapture } from "$lib/api";

  let mode = $state<"task" | "note">("task");
  let text = $state("");
  let input: HTMLInputElement | undefined;
  let busy = $state(false);

  onMount(() => {
    const html = document.documentElement, body = document.body;
    const prevHtml = html.style.background, prevBody = body.style.background;
    html.style.background = "transparent";
    body.style.background = "transparent";
    input?.focus();
    return () => { html.style.background = prevHtml; body.style.background = prevBody; };
  });

  async function close() { try { await hideCapture(); } catch { /* not in Tauri */ } }

  async function save() {
    const t = text.trim();
    if (!t) { await close(); return; }
    busy = true;
    try {
      if (mode === "task") {
        await addTask(t);
      } else {
        const id = await addNote();
        await updateNote({ id, title: "", body: t, color: "amber", due: null });
      }
      text = "";
      await close();
    } finally { busy = false; }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); close(); }
  }
  function pick(m: "task" | "note") { mode = m; input?.focus(); }
</script>

<svelte:window onkeydown={onKey} />

<div class="cap">
  <div class="bar" data-tauri-drag-region>
    <div class="seg">
      <button class:on={mode === "task"} onclick={() => pick("task")}>Task</button>
      <button class:on={mode === "note"} onclick={() => pick("note")}>Note</button>
    </div>
    <button class="x" onclick={close} title="Close" aria-label="Close capture">×</button>
  </div>
  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    <input bind:this={input} bind:value={text} disabled={busy}
      placeholder={mode === "task" ? "Add a task…" : "Jot a note…"} aria-label="Capture text" />
  </form>
  <div class="hint">Enter to save · Esc to close</div>
</div>

<style>
  .cap { font-family: var(--font-body); color: var(--text); height: 100vh; box-sizing: border-box;
    display: flex; flex-direction: column; gap: 10px; padding: 12px 14px;
    background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); }
  .bar { display: flex; align-items: center; justify-content: space-between; cursor: move; }
  .seg { display: flex; gap: 4px; }
  .seg button { font: inherit; font-size: 12px; padding: 5px 12px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--muted); cursor: pointer; }
  .seg button.on { background: var(--accent); color: var(--accent-contrast); border-color: var(--accent); }
  .x { border: none; background: none; color: var(--muted); font-size: 18px; line-height: 1; cursor: pointer; }
  .x:hover { color: var(--text); }
  input { width: 100%; box-sizing: border-box; font: inherit; font-size: 15px; padding: 10px 12px;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  input:focus { outline: 1px solid var(--accent); }
  .hint { font-size: 11px; color: var(--muted); text-align: right; }
</style>
