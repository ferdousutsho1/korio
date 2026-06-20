<script lang="ts">
  import { onMount } from "svelte";
  import { listApps, runningApps, addApp, removeApp, colorFor,
    type App, type RunningApp } from "$lib/api";

  let apps = $state<App[]>([]);
  let running = $state<RunningApp[]>([]);
  let picking = $state(false);

  async function refresh() { apps = await listApps(); }
  onMount(refresh);

  async function openPicker() {
    running = await runningApps();
    picking = true;
  }

  async function pick(r: RunningApp) {
    const name = r.title.length > 40 ? r.exe_name.replace(/\.exe$/i, "") : r.title;
    await addApp({
      display_name: name, exe_name: r.exe_name, kind: "neutral",
      color: colorFor(apps.length),
    });
    picking = false;
    await refresh();
  }

  async function drop(a: App) { await removeApp(a.id); await refresh(); }

  function onOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) picking = false;
  }
</script>

<div class="wrap">
  <div class="head">
    <p class="sub">Apps Korio actively times. Time counts only while one is focused and you're active.</p>
    <button class="add" onclick={openPicker}>+ Add app</button>
  </div>

  {#if apps.length === 0}
    <div class="empty">No apps yet. Click <strong>Add app</strong> and pick from what's running.</div>
  {:else}
    <ul class="list">
      {#each apps as a}
        <li>
          <span class="dot" style={`background:${a.color}`}></span>
          <span class="name">{a.display_name}</span>
          <span class="exe">{a.exe_name}</span>
          <button class="x" onclick={() => drop(a)} title="Remove" aria-label="Remove {a.display_name}">×</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if picking}
  <div class="modal" role="presentation" onclick={onOverlayClick}>
    <div class="sheet">
      <h3>Pick from running apps</h3>
      <ul class="run">
        {#each running as r}
          <button onclick={() => pick(r)}>
            <span class="rexe">{r.exe_name}</span>
            <span class="rtitle">{r.title}</span>
          </button>
        {/each}
      </ul>
    </div>
  </div>
{/if}

<style>
  .head { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 18px; }
  .sub { color: var(--muted); margin: 0; max-width: 520px; }
  .add { background: var(--accent); color: var(--accent-contrast); border: none;
    padding: 9px 14px; border-radius: var(--radius-sm); cursor: pointer; font: inherit; }
  .empty { color: var(--muted); border: 1px dashed var(--line); border-radius: var(--radius);
    padding: 40px; text-align: center; }
  .list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; }
  .list li { display: flex; align-items: center; gap: 12px; background: var(--surface);
    border: 1px solid var(--line); border-radius: var(--radius); padding: 12px 14px; }
  .dot { width: 10px; height: 10px; border-radius: 3px; }
  .name { font-weight: 600; }
  .exe { color: var(--muted); font-size: 12px; }
  .x { margin-left: auto; border: none; background: transparent; color: var(--muted);
    font-size: 20px; cursor: pointer; line-height: 1; }
  .modal { position: fixed; inset: 0; background: rgba(0,0,0,.35);
    display: flex; align-items: center; justify-content: center; }
  .sheet { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius);
    width: 460px; max-height: 70vh; overflow: auto; padding: 20px; }
  .sheet h3 { font-family: var(--font-display); margin: 0 0 14px; }
  .run { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 4px; }
  .run button { display: flex; flex-direction: column; align-items: flex-start; gap: 2px;
    width: 100%; text-align: left; background: transparent; border: 1px solid transparent;
    border-radius: var(--radius-sm); padding: 10px; cursor: pointer; color: var(--text); font: inherit; }
  .run button:hover { border-color: var(--line); background: var(--bg); }
  .rexe { font-weight: 600; font-size: 13px; }
  .rtitle { color: var(--muted); font-size: 12px; }
</style>
