<script lang="ts">
  import { onMount } from "svelte";
  import { listReminders, addReminder, updateReminder, setReminderDone, snoozeReminder,
    deleteReminder, type Reminder, type RepeatRule } from "$lib/api";
  import { toLocalInput, fromLocalInput, formatWhen, REPEAT_LABELS, remindersTick } from "$lib/reminders";

  let items = $state<Reminder[]>([]);
  let loading = $state(true);
  let err = $state("");

  // New-reminder form. Defaults to the next round hour so the common case is one click.
  function defaultWhen(): string {
    const d = new Date();
    d.setMinutes(0, 0, 0);
    d.setHours(d.getHours() + 1);
    return toLocalInput(Math.floor(d.getTime() / 1000));
  }
  let title = $state("");
  let when = $state(defaultWhen());
  let repeat = $state<RepeatRule>("once");

  async function refresh() {
    try { items = await listReminders(); err = ""; }
    catch (e) { err = String(e); }
    loading = false;
  }
  onMount(refresh);
  // The scheduler snoozes/completes reminders behind our back — re-read when it does.
  $effect(() => { $remindersTick; refresh(); });

  async function create() {
    const t = title.trim();
    const at = fromLocalInput(when);
    if (!t || at === null) return;
    try {
      await addReminder(t, at, repeat);
      title = ""; when = defaultWhen(); repeat = "once"; err = "";
      await refresh();
    } catch (e) { err = String(e); }
  }

  async function saveTitle(r: Reminder, value: string) {
    const t = value.trim();
    if (!t || t === r.title) return;
    await updateReminder(r.id, t, r.at_ts, r.repeat_rule);
    await refresh();
  }
  async function saveWhen(r: Reminder, value: string) {
    const at = fromLocalInput(value);
    if (at === null || at === r.at_ts) return;
    await updateReminder(r.id, r.title, at, r.repeat_rule);
    await refresh();
  }
  async function saveRepeat(r: Reminder, value: string) {
    await updateReminder(r.id, r.title, r.at_ts, value as RepeatRule);
    await refresh();
  }
  async function toggleDone(r: Reminder) { await setReminderDone(r.id, !r.done); await refresh(); }
  async function later(r: Reminder, minutes: number) {
    await snoozeReminder(r.id, Math.floor(Date.now() / 1000) + minutes * 60);
    await refresh();
  }
  async function drop(r: Reminder) { await deleteReminder(r.id); await refresh(); }

  let now = $state(Math.floor(Date.now() / 1000));
  onMount(() => {
    const t = setInterval(() => (now = Math.floor(Date.now() / 1000)), 30_000);
    return () => clearInterval(t);
  });

  let pending = $derived(items.filter((r) => !r.done));
  let done = $derived(items.filter((r) => r.done));
</script>

<div class="wrap">
  <div class="head">
    <p class="sub">Nudges on your schedule. Set a date and time, repeat them, and snooze
      anything you're not ready for.</p>
  </div>

  <div class="new">
    <input class="t" bind:value={title} placeholder="Remind me to…" aria-label="Reminder text"
      onkeydown={(e) => { if (e.key === "Enter") create(); }} />
    <input class="w" type="datetime-local" bind:value={when} aria-label="Reminder date and time" />
    <select class="r" bind:value={repeat} aria-label="Repeat">
      {#each REPEAT_LABELS as o}<option value={o.id}>{o.label}</option>{/each}
    </select>
    <button class="add" onclick={create} disabled={!title.trim()}>Add reminder</button>
  </div>
  {#if err}<p class="err">{err}</p>{/if}

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if items.length === 0}
    <div class="empty box">No reminders yet. Add one above.</div>
  {:else}
    <ul class="list">
      {#each pending as r (r.id)}
        {@const overdue = r.at_ts <= now}
        <li class:overdue>
          <button class="check" onclick={() => toggleDone(r)} aria-label={`Complete ${r.title}`}>○</button>
          <span class="body">
            <input class="title" value={r.title} aria-label="Reminder text"
              onchange={(e) => saveTitle(r, e.currentTarget.value)} />
            <span class="meta">
              <input class="when" type="datetime-local" value={toLocalInput(r.at_ts)}
                aria-label={`When to remind for ${r.title}`}
                onchange={(e) => saveWhen(r, e.currentTarget.value)} />
              <select class="rep" value={r.repeat_rule} aria-label={`Repeat for ${r.title}`}
                onchange={(e) => saveRepeat(r, e.currentTarget.value)}>
                {#each REPEAT_LABELS as o}<option value={o.id}>{o.label}</option>{/each}
              </select>
              <span class="rel" class:due={overdue}>{overdue ? "Due — " : ""}{formatWhen(r.at_ts, now)}</span>
            </span>
          </span>
          <span class="acts">
            <button onclick={() => later(r, 60)} title="Push this out by an hour">+1h</button>
            <button onclick={() => later(r, 1440)} title="Push this out by a day">+1d</button>
            <button class="x" onclick={() => drop(r)} aria-label={`Delete ${r.title}`}>×</button>
          </span>
        </li>
      {/each}

      {#if done.length}
        <li class="sep">Completed</li>
        {#each done as r (r.id)}
          <li class="is-done">
            <button class="check on" onclick={() => toggleDone(r)} aria-label={`Reopen ${r.title}`}>●</button>
            <span class="body"><span class="title flat">{r.title}</span>
              <span class="meta"><span class="rel">{formatWhen(r.at_ts, now)}</span></span></span>
            <span class="acts"><button class="x" onclick={() => drop(r)} aria-label={`Delete ${r.title}`}>×</button></span>
          </li>
        {/each}
      {/if}
    </ul>
  {/if}
</div>

<style>
  .wrap { max-width: 760px; }
  .sub { color: var(--muted); margin: 0 0 18px; max-width: 560px; }
  .new { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 18px; }
  .new input, .new select { font: inherit; font-size: 13px; padding: 8px 10px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .new .t { flex: 1; min-width: 200px; }
  .add { background: var(--accent); color: var(--accent-contrast); border: none;
    padding: 9px 14px; border-radius: var(--radius-sm); cursor: pointer; font: inherit; font-size: 13px; }
  .add:disabled { opacity: .5; cursor: default; }
  .err { color: var(--warn-close); font-size: 12px; margin: 0 0 12px; }
  .empty { color: var(--muted); font-size: 13px; }
  .empty.box { border: 1px dashed var(--line); border-radius: var(--radius); padding: 40px; text-align: center; }
  .list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; }
  .list li { display: flex; align-items: flex-start; gap: 12px; background: var(--surface);
    border: 1px solid var(--line); border-radius: var(--radius); padding: 12px 14px; }
  .list li.overdue { border-color: var(--warn-close); }
  .list li.sep { background: none; border: none; padding: 14px 0 2px; color: var(--muted);
    font-size: 10px; letter-spacing: 2px; text-transform: uppercase; }
  .check { border: none; background: transparent; color: var(--muted); font-size: 15px;
    cursor: pointer; line-height: 1.6; padding: 0; }
  .check:hover, .check.on { color: var(--accent); }
  .body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
  .title { font: inherit; font-size: 14px; font-weight: 600; width: 100%; padding: 2px 4px;
    border: 1px solid transparent; border-radius: var(--radius-sm); background: transparent; color: var(--text); }
  .title:hover, .title:focus { border-color: var(--line); background: var(--bg); outline: none; }
  .title.flat { padding: 2px 4px; color: var(--muted); text-decoration: line-through; }
  .meta { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .when, .rep { font: inherit; font-size: 11px; padding: 3px 6px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--muted); }
  .rel { font-size: 11px; color: var(--muted); }
  .rel.due { color: var(--warn-close); font-weight: 600; }
  .acts { display: flex; align-items: center; gap: 4px; }
  .acts button { font: inherit; font-size: 11px; padding: 4px 8px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--muted); cursor: pointer; }
  .acts button:hover { color: var(--text); border-color: var(--accent); }
  .acts .x { border: none; background: transparent; font-size: 18px; line-height: 1; padding: 0 4px; }
  .is-done { opacity: .65; }
</style>
