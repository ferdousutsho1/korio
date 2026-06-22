<script lang="ts">
  import { listNotes, type Note } from "$lib/api";
  let notes = $state<Note[]>([]);
  $effect(() => { listNotes().then((n) => (notes = n)); });
</script>

<div class="card">
  <div class="label">Notes</div>
  {#if notes.length === 0}
    <div class="muted">No notes yet.</div>
  {:else}
    <ul>
      {#each notes.slice(0, 4) as n}<li>{n.title || "Untitled"}</li>{/each}
    </ul>
  {/if}
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  ul { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
  .muted { color: var(--muted); font-size: 13px; }
</style>
