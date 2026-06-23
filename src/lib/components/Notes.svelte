<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listNotes, addNote, updateNote, deleteNote, setNoteSize, type Note } from "$lib/api";
  import { NOTE_COLORS, isOverdue } from "$lib/notes";

  let notes = $state<Note[]>([]);
  const timers = new Map<number, ReturnType<typeof setTimeout>>();

  async function refresh() { notes = await listNotes(); }
  onMount(refresh);
  onDestroy(() => { for (const t of timers.values()) clearTimeout(t); for (const t of sizeTimers.values()) clearTimeout(t); });

  async function create() {
    const id = await addNote();
    await refresh();
    // focus the new card's title (it sorts first)
    queueMicrotask(() => {
      const el = document.querySelector<HTMLInputElement>(`[data-note="${id}"] .title`);
      el?.focus();
    });
  }

  // Debounced save: persist 500ms after the last edit to this note.
  function scheduleSave(note: Note) {
    const existing = timers.get(note.id);
    if (existing) clearTimeout(existing);
    timers.set(note.id, setTimeout(() => {
      timers.delete(note.id);
      updateNote({ id: note.id, title: note.title, body: note.body, color: note.color, due: note.due });
    }, 500));
  }

  async function remove(note: Note) {
    const t = timers.get(note.id);
    if (t) { clearTimeout(t); timers.delete(note.id); }
    await deleteNote(note.id);
    await refresh();
  }

  const sizeTimers = new Map<number, ReturnType<typeof setTimeout>>();
  function persistSize(note: Note, w: number, h: number) {
    note.width = w; note.height = h;
    const ex = sizeTimers.get(note.id); if (ex) clearTimeout(ex);
    sizeTimers.set(note.id, setTimeout(() => {
      sizeTimers.delete(note.id);
      setNoteSize(note.id, Math.round(w), Math.round(h));
    }, 400));
  }
  // Svelte action: observe the card's size and persist changes (after first paint).
  function resizable(node: HTMLElement, note: Note) {
    let first = true;
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) {
        const w = e.contentRect.width, h = e.contentRect.height;
        if (first) { first = false; continue; }
        persistSize(note, w, h);
      }
    });
    ro.observe(node);
    return { destroy() { ro.disconnect(); } };
  }
</script>

<div class="notes">
  <div class="bar">
    <button class="new" onclick={create}>+ New note</button>
  </div>

  {#if notes.length === 0}
    <div class="empty">No notes yet. Create one to jot something down.</div>
  {:else}
    <div class="grid">
      {#each notes as note (note.id)}
        <article class="card" data-note={note.id} use:resizable={note}
          style={`background: var(--note-${note.color}); width:${note.width ?? 240}px; ${note.height ? `height:${note.height}px;` : ""}`}>
          <header class="head">
            <input class="title" placeholder="Title" bind:value={note.title}
              oninput={() => scheduleSave(note)} aria-label="Note title" />
            <button class="x" onclick={() => remove(note)} title="Delete" aria-label="Delete note">×</button>
          </header>
          <textarea class="body" rows="3" placeholder="Write something…" bind:value={note.body}
            oninput={() => scheduleSave(note)} aria-label="Note body"></textarea>
          <footer class="foot">
            <div class="swatches" role="group" aria-label="Note color">
              {#each NOTE_COLORS as c}
                <button class="sw" class:on={note.color === c} style="background: var(--note-{c});"
                  title={c} aria-label={c} aria-pressed={note.color === c}
                  onclick={() => { note.color = c; scheduleSave(note); }}></button>
              {/each}
            </div>
            <input class="due" type="date" class:overdue={isOverdue(note.due)} value={note.due ?? ""}
              onchange={(e) => { note.due = e.currentTarget.value || null; scheduleSave(note); }} aria-label="Due date" />
          </footer>
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .notes { display: flex; flex-direction: column; gap: 16px; }
  .bar { display: flex; }
  .new { font: inherit; padding: 9px 16px; border: none; border-radius: var(--radius-sm);
    background: var(--accent); color: var(--accent-contrast); cursor: pointer; }
  .empty { color: var(--muted); border: 1px dashed var(--line); border-radius: var(--radius);
    padding: 40px; text-align: center; }
  /* free-flow: cards keep their own size and wrap */
  .grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-start; }
  .card { break-inside: avoid; display: flex; flex-direction: column; gap: 8px;
    padding: 12px; border-radius: var(--radius); box-sizing: border-box;
    border: 1px solid color-mix(in srgb, var(--text) 12%, transparent);
    box-shadow: 0 1px 2px color-mix(in srgb, var(--text) 8%, transparent); color: var(--text);
    resize: both; overflow: hidden; min-width: 180px; min-height: 120px; }
  .head { display: flex; align-items: center; gap: 8px; }
  .title { flex: 1; font: inherit; font-weight: 600; font-size: 14px; border: none;
    background: none; color: var(--text); padding: 2px 0; }
  .title::placeholder, .body::placeholder { color: color-mix(in srgb, var(--text) 45%, transparent); }
  .x { border: none; background: none; color: var(--text); opacity: .5; font-size: 18px;
    line-height: 1; cursor: pointer; }
  .x:hover { opacity: 1; }
  .body { flex: 1; font: inherit; font-size: 13px; line-height: 1.5; border: none; background: none;
    color: var(--text); resize: none; min-height: 0; padding: 0; }
  .title:focus, .body:focus, .due:focus { outline: none; }
  .foot { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .swatches { display: flex; gap: 5px; }
  .sw { width: 16px; height: 16px; border-radius: 50%; cursor: pointer;
    border: 1px solid color-mix(in srgb, var(--text) 20%, transparent); padding: 0; }
  .sw.on { outline: 2px solid var(--text); outline-offset: 1px; }
  .due { font: inherit; font-size: 11px; color: var(--text); background: none;
    border: 1px solid color-mix(in srgb, var(--text) 18%, transparent); border-radius: 6px; padding: 2px 4px; }
  .due.overdue { color: var(--note-overdue); border-color: var(--note-overdue); }
</style>
