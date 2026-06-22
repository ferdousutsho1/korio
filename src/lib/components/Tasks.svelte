<script lang="ts">
  import { listTasks, addTask, setTaskDone, updateTaskTitle, deleteTask, clearDoneTasks, type Task } from "$lib/api";
  import { dayStartLocal, isoDate } from "$lib/ranges";

  let tasks = $state<Task[]>([]);
  let newTitle = $state("");
  let doneCount = $derived(tasks.filter((t) => t.done).length);

  let day = $state<number>(dayStartLocal(new Date()));   // local midnight, unix secs
  const DAY = 86400;
  const isToday = $derived(day === dayStartLocal(new Date()));

  async function refresh() { tasks = await listTasks(day, day + DAY); }
  $effect(() => { day; refresh(); });

  async function add() {
    const t = newTitle.trim();
    if (!t) return;
    newTitle = "";
    // file under the selected day; use real "now" when viewing today
    await addTask(t, isToday ? undefined : day);
    await refresh();
  }
  async function toggle(task: Task) { await setTaskDone(task.id, !task.done); await refresh(); }
  async function rename(task: Task, title: string) {
    const t = title.trim();
    if (!t || t === task.title) return;
    await updateTaskTitle(task.id, t);
    await refresh();
  }
  async function remove(task: Task) { await deleteTask(task.id); await refresh(); }
  async function clearDone() { await clearDoneTasks(); await refresh(); }
</script>

<div class="tasks">
  <div class="daybar">
    <button class="nav" onclick={() => (day -= DAY)} aria-label="Previous day">‹</button>
    <input class="day" type="date" value={isoDate(day)}
      onchange={(e) => { const [y,m,d] = e.currentTarget.value.split('-').map(Number); day = dayStartLocal(new Date(y, m-1, d)); }}
      aria-label="Task date" />
    <button class="nav" onclick={() => (day += DAY)} disabled={isToday} aria-label="Next day">›</button>
    {#if !isToday}<button class="today" onclick={() => (day = dayStartLocal(new Date()))}>Today</button>{/if}
  </div>
  <form class="add" onsubmit={(e) => { e.preventDefault(); add(); }}>
    <input bind:value={newTitle} placeholder="Add a task and press Enter…" aria-label="New task" />
    <button type="submit" class="addbtn">Add</button>
  </form>

  {#if tasks.length === 0}
    <div class="empty">No tasks for this day. Add one above.</div>
  {:else}
    <ul class="list">
      {#each tasks as task (task.id)}
        <li class:done={task.done}>
          <button class="check" class:on={task.done} role="checkbox" aria-checked={task.done}
            aria-label={task.done ? "Mark not done" : "Mark done"} onclick={() => toggle(task)}>
            {#if task.done}✓{/if}
          </button>
          <input class="title" value={task.title} aria-label="Task title"
            onchange={(e) => rename(task, e.currentTarget.value)} />
          <button class="x" onclick={() => remove(task)} title="Delete" aria-label="Delete task">×</button>
        </li>
      {/each}
    </ul>
    <div class="foot">
      <span>{doneCount} of {tasks.length} done</span>
      {#if doneCount > 0}<button class="clear" onclick={clearDone}>Clear completed</button>{/if}
    </div>
  {/if}
</div>

<style>
  .tasks { max-width: 620px; display: flex; flex-direction: column; gap: 16px; }
  .add { display: flex; gap: 8px; }
  .add input { flex: 1; font: inherit; font-size: 14px; padding: 10px 12px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--text); }
  .addbtn { font: inherit; padding: 10px 18px; border: none; border-radius: var(--radius-sm);
    background: var(--accent); color: var(--accent-contrast); cursor: pointer; }
  .empty { color: var(--muted); border: 1px dashed var(--line); border-radius: var(--radius); padding: 40px; text-align: center; }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
  .list li { display: flex; align-items: center; gap: 12px; background: var(--surface);
    border: 1px solid var(--line); border-radius: var(--radius); padding: 10px 12px; }
  .check { width: 22px; height: 22px; flex-shrink: 0; border: 1.5px solid var(--line); border-radius: 6px;
    background: var(--bg); color: var(--accent-contrast); cursor: pointer; font-size: 13px; line-height: 1;
    display: flex; align-items: center; justify-content: center; padding: 0; }
  .check.on { background: var(--accent); border-color: var(--accent); }
  .title { flex: 1; font: inherit; font-size: 14px; border: none; background: none; color: var(--text); padding: 4px; }
  .title:focus { outline: 1px solid var(--accent); border-radius: 4px; }
  li.done .title { text-decoration: line-through; color: var(--muted); }
  .x { border: none; background: none; color: var(--muted); font-size: 20px; cursor: pointer; line-height: 1; }
  .foot { display: flex; justify-content: space-between; align-items: center; color: var(--muted); font-size: 13px; }
  .clear { font: inherit; font-size: 12px; padding: 5px 10px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--muted); cursor: pointer; }
  .daybar { display: flex; align-items: center; gap: 8px; margin-bottom: 14px; }
  .daybar .nav { width: 30px; height: 30px; border: 1px solid var(--line); border-radius: var(--radius-sm);
    background: var(--surface); color: var(--text); cursor: pointer; line-height: 1; }
  .daybar .nav:disabled { opacity: .4; cursor: default; }
  .daybar .day { font: inherit; font-size: 13px; padding: 6px 8px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--text); color-scheme: light dark; }
  .daybar .today { font: inherit; font-size: 12px; padding: 6px 10px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--muted); cursor: pointer; }
</style>
