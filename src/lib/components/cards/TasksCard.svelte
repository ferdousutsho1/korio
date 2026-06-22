<script lang="ts">
  import { listTasks, type Task } from "$lib/api";
  import { dayStartLocal } from "$lib/ranges";
  let tasks = $state<Task[]>([]);
  const day = dayStartLocal(new Date());
  $effect(() => { listTasks(day, day + 86400).then((t) => (tasks = t)); });
  let open = $derived(tasks.filter((t) => !t.done));
</script>

<div class="card">
  <div class="label">Tasks today</div>
  {#if open.length === 0}
    <div class="muted">Nothing open. 🎉</div>
  {:else}
    <ul>
      {#each open.slice(0, 5) as t}<li>{t.title}</li>{/each}
    </ul>
    {#if open.length > 5}<div class="muted">+{open.length - 5} more</div>{/if}
  {/if}
</div>

<style>
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  ul { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
  .muted { color: var(--muted); font-size: 13px; }
</style>
