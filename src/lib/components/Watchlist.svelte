<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listApps, runningApps, addApp, removeApp, setAppLimit, colorFor,
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

  async function browseForExe() {
    const path = await open({ multiple: false, filters: [{ name: "Programs", extensions: ["exe"] }] });
    if (typeof path !== "string") return;
    const file = path.replace(/^.*[\\/]/, "");   // "Code.exe"
    const name = file.replace(/\.exe$/i, "");      // "Code"
    await addApp({ display_name: name, exe_name: file, kind: "neutral", color: colorFor(apps.length) });
    await refresh();
  }

  async function drop(a: App) { await removeApp(a.id); await refresh(); }

  async function commitLimit(a: App, minutes: number, action: string) {
    await setAppLimit(a.id, Math.max(0, Math.round(minutes)) * 60, action);
    await refresh();
  }

  function onOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) picking = false;
  }
</script>

<div class="wrap">
  <div class="head">
    <p class="sub">Apps Korio actively times. Time counts only while one is focused and you're active.</p>
    <div class="addbtns">
      <button class="add" onclick={openPicker}>+ Add app</button>
      <button class="add ghost" onclick={browseForExe}>Browse for .exe</button>
    </div>
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
          <span class="limit">
            <input class="cap" type="number" min="0" step="5" value={Math.round(a.daily_cap_seconds / 60)}
              title="Daily limit (minutes, 0 = off)" aria-label={`Daily limit minutes for ${a.display_name}`}
              onchange={(e) => commitLimit(a, +e.currentTarget.value, a.limit_action)} />
            <span class="unit">min</span>
            <button class="act" class:close={a.limit_action === "close"}
              title={a.limit_action === "close" ? "Auto-close when over limit" : "Warn when over limit"}
              onclick={() => commitLimit(a, Math.round(a.daily_cap_seconds / 60), a.limit_action === "close" ? "warn" : "close")}>
              {a.limit_action === "close" ? "Auto-close" : "Warn"}
            </button>
          </span>
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
  .addbtns { display: flex; gap: 8px; }
  .ghost { background: var(--surface); color: var(--text); border: 1px solid var(--line); }
  .empty { color: var(--muted); border: 1px dashed var(--line); border-radius: var(--radius);
    padding: 40px; text-align: center; }
  .list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; }
  .list li { display: flex; align-items: center; gap: 12px; background: var(--surface);
    border: 1px solid var(--line); border-radius: var(--radius); padding: 12px 14px; }
  .dot { width: 10px; height: 10px; border-radius: 3px; }
  .name { font-weight: 600; }
  .exe { color: var(--muted); font-size: 12px; }
  .x { border: none; background: transparent; color: var(--muted);
    font-size: 20px; cursor: pointer; line-height: 1; }
  .limit { margin-left: auto; display: flex; align-items: center; gap: 6px; }
  .cap { width: 52px; font: inherit; font-size: 12px; padding: 4px 6px; text-align: right;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .unit { color: var(--muted); font-size: 11px; }
  .act { font: inherit; font-size: 11px; padding: 4px 8px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--surface); color: var(--muted); cursor: pointer; }
  .act.close { border-color: var(--accent); color: var(--accent); }
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
