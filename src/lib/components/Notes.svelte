<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listNotes, addNote, updateNote, deleteNote, type Note } from "$lib/api";
  import { NOTE_COLORS, isOverdue } from "$lib/notes";

  let notes = $state<Note[]>([]);
  const timers = new Map<number, ReturnType<typeof setTimeout>>();

  async function refresh() { notes = await listNotes(); }
  onMount(refresh);
  onDestroy(() => { for (const t of timers.values()) clearTimeout(t); });

  async function create() {
    const id = await addNote();
    await refresh();
    // focus the new card's title (it sorts first)
    queueMicrotask(() => {
      const el = document.querySelector<HTMLInputElement>(`[data-note="${id}"] .title`);
      el?.focus();
    });
  }

  // Debounced save: mutate local note, persist 500ms after the last edit.
  function edit(note: Note, patch: Partial<Note>) {
    Object.assign(note, patch);
    notes = notes; // trigger reactivity
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
        <article class="card" data-note={note.id} style="background: var(--note-{note.color});">
          <header class="head">
            <input class="title" placeholder="Title" value={note.title}
              oninput={(e) => edit(note, { title: e.currentTarget.value })} aria-label="Note title" />
            <button class="x" onclick={() => remove(note)} title="Delete" aria-label="Delete note">×</button>
          </header>
          <textarea class="body" rows="3" placeholder="Write something…" value={note.body}
            oninput={(e) => edit(note, { body: e.currentTarget.value })} aria-label="Note body"></textarea>
          <footer class="foot">
            <div class="swatches" role="group" aria-label="Note color">
              {#each NOTE_COLORS as c}
                <button class="sw" class:on={note.color === c} style="background: var(--note-{c});"
                  title={c} aria-label={c} aria-pressed={note.color === c}
                  onclick={() => edit(note, { color: c })}></button>
              {/each}
            </div>
            <input class="due" type="date" class:overdue={isOverdue(note.due)} value={note.due ?? ""}
              onchange={(e) => edit(note, { due: e.currentTarget.value || null })} aria-label="Due date" />
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
  /* masonry: balanced columns, cards never split */
  .grid { column-width: 240px; column-gap: 16px; }
  .card { break-inside: avoid; display: flex; flex-direction: column; gap: 8px;
    margin: 0 0 16px; padding: 12px; border-radius: var(--radius);
    border: 1px solid color-mix(in srgb, var(--text) 12%, transparent);
    box-shadow: 0 1px 2px color-mix(in srgb, var(--text) 8%, transparent); color: var(--text); }
  .head { display: flex; align-items: center; gap: 8px; }
  .title { flex: 1; font: inherit; font-weight: 600; font-size: 14px; border: none;
    background: none; color: var(--text); padding: 2px 0; }
  .title::placeholder, .body::placeholder { color: color-mix(in srgb, var(--text) 45%, transparent); }
  .x { border: none; background: none; color: var(--text); opacity: .5; font-size: 18px;
    line-height: 1; cursor: pointer; }
  .x:hover { opacity: 1; }
  .body { font: inherit; font-size: 13px; line-height: 1.5; border: none; background: none;
    color: var(--text); resize: vertical; min-height: 48px; padding: 0; }
  .title:focus, .body:focus, .due:focus { outline: none; }
  .foot { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .swatches { display: flex; gap: 5px; }
  .sw { width: 16px; height: 16px; border-radius: 50%; cursor: pointer;
    border: 1px solid color-mix(in srgb, var(--text) 20%, transparent); padding: 0; }
  .sw.on { outline: 2px solid var(--text); outline-offset: 1px; }
  .due { font: inherit; font-size: 11px; color: var(--text); background: none;
    border: 1px solid color-mix(in srgb, var(--text) 18%, transparent); border-radius: 6px; padding: 2px 4px; }
  .due.overdue { color: #C2410C; border-color: #C2410C; }
</style>
