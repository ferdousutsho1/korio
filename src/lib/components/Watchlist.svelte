<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listApps, runningApps, addApp, removeApp, setAppLimit, launchApp, colorFor, renameApp,
    listCategories, addCategory, updateCategory, deleteCategory, setAppCategory, usageToday,
    type App, type RunningApp, type Category } from "$lib/api";
  import { formatDuration } from "$lib/format";

  let apps = $state<App[]>([]);
  let running = $state<RunningApp[]>([]);
  let picking = $state(false);
  let categories = $state<Category[]>([]);
  let managing = $state(false);
  let usedById = $state<Record<number, number>>({});
  async function refreshUsage() {
    try {
      const slices = await usageToday();
      usedById = Object.fromEntries(slices.map((s) => [s.app_id, s.seconds]));
    } catch { /* keep last */ }
  }
  let usagePoll: ReturnType<typeof setInterval> | null = null;

  async function refresh() { apps = await listApps(); }
  async function refreshCats() { categories = await listCategories(); }
  onMount(async () => {
    await refresh(); await refreshCats(); await refreshUsage();
    usagePoll = setInterval(refreshUsage, 30_000);
  });
  onDestroy(() => { if (usagePoll) clearInterval(usagePoll); });

  async function assignCategory(a: App, value: string) {
    const id = value === "" ? null : Number(value);
    try { await setAppCategory(a.id, id); }
    catch (e) { console.error("setAppCategory failed", e); }
    await refresh();
  }

  let newName = $state("");
  let newColor = $state("#3A6EA5");
  let newNature = $state("neutral");
  let catErr = $state("");

  function friendlyCatErr(e: unknown): string {
    const s = String(e);
    return s.includes("UNIQUE") ? "A category with that name already exists." : s;
  }

  async function createCat() {
    const n = newName.trim();
    if (!n) return;
    try { await addCategory(n, newColor, newNature); catErr = ""; newName = ""; await refreshCats(); }
    catch (e) { catErr = friendlyCatErr(e); }
  }

  async function saveCat(c: Category, patch: Partial<Category>) {
    const next = { ...c, ...patch };
    try { await updateCategory(c.id, next.name.trim() || c.name, next.color, next.nature); catErr = ""; await refreshCats(); await refresh(); }
    catch (e) { catErr = friendlyCatErr(e); }
  }

  async function dropCat(c: Category) {
    try { await deleteCategory(c.id); catErr = ""; await refreshCats(); await refresh(); }
    catch (e) { catErr = friendlyCatErr(e); }
  }

  async function openPicker() {
    running = await runningApps();
    picking = true;
  }

  async function pick(r: RunningApp) {
    const name = r.title.length > 40 ? r.exe_name.replace(/\.exe$/i, "") : r.title;
    await addApp({
      display_name: name, exe_name: r.exe_name, kind: "neutral",
      color: colorFor(apps.length), exe_path: r.path || null,
    });
    picking = false;
    await refresh();
  }

  async function browseForExe() {
    const path = await open({ multiple: false, filters: [{ name: "Programs", extensions: ["exe"] }] });
    if (typeof path !== "string") return;
    const file = path.replace(/^.*[\\/]/, "");   // "Code.exe"
    const name = file.replace(/\.exe$/i, "");      // "Code"
    await addApp({ display_name: name, exe_name: file, kind: "neutral", color: colorFor(apps.length), exe_path: path });
    await refresh();
  }

  async function drop(a: App) { await removeApp(a.id); await refresh(); }

  /** Display-only rename — tracking keeps using the app's exe name. */
  async function commitName(a: App, value: string) {
    const next = value.trim();
    if (!next || next === a.display_name) { await refresh(); return; }
    await renameApp(a.id, next);
    await refresh();
  }

  async function commitLimit(a: App, minutes: number, action: string) {
    await setAppLimit(a.id, Math.max(0, Math.round(minutes)) * 60, action);
    await refresh(); await refreshUsage();
  }

  function onOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) picking = false;
  }
</script>

<div class="wrap">
  <div class="head">
    <p class="sub">Apps Korio actively times. Time counts only while one is focused and you're active.
      <br /><span class="hint">Daily limits reset each day — set minutes per app; 0 = off.</span></p>
    <div class="addbtns">
      <button class="add" onclick={openPicker}>+ Add app</button>
      <button class="add ghost" onclick={browseForExe}>Browse for .exe</button>
      <button class="add ghost" onclick={() => (managing = true)}>Manage categories</button>
    </div>
  </div>

  {#if apps.length === 0}
    <div class="empty">No apps yet. Click <strong>Add app</strong> and pick from what's running.</div>
  {:else}
    <ul class="list">
      {#each apps as a}
        <li>
          <span class="dot" style={`background:${a.color}`}></span>
          <span class="who">
            <input class="name" value={a.display_name} aria-label={`Name for ${a.exe_name}`}
              title="Rename — tracking still follows {a.exe_name}"
              onchange={(e) => commitName(a, e.currentTarget.value)} />
            <span class="exe">{a.exe_name}</span>
            {#if a.daily_cap_seconds > 0}
              {@const used = usedById[a.id] ?? 0}
              {@const over = used > a.daily_cap_seconds}
              <span class="meter"><span class="mfill" class:over
                style={`width:${Math.min(used / a.daily_cap_seconds, 1) * 100}%;${over ? "" : `background:${a.color}`}`}></span></span>
              {#if over}
                <span class="overtxt">Over limit — {formatDuration(used - a.daily_cap_seconds)} past {formatDuration(a.daily_cap_seconds)}</span>
              {/if}
            {/if}
          </span>
          <button class="launch" onclick={() => launchApp(a.id)}
            title="Launch {a.display_name}" aria-label="Launch {a.display_name}">▷</button>
          <select class="cat" aria-label={`Category for ${a.display_name}`}
            value={a.category_id == null ? "" : String(a.category_id)}
            onchange={(e) => assignCategory(a, e.currentTarget.value)}>
            <option value="">Uncategorized</option>
            {#each categories as c (c.id)}<option value={String(c.id)}>{c.name}</option>{/each}
          </select>
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

{#if managing}
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="cat-modal-title" tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) managing = false; }}
    onkeydown={(e) => { if (e.key === "Escape") managing = false; }}>
    <div class="sheet">
      <h3 id="cat-modal-title">Categories</h3>
      {#if catErr}<p class="err">{catErr}</p>{/if}
      <ul class="cats">
        {#each categories as c (c.id)}
          <li>
            <input class="cname" value={c.name} aria-label="Category name"
              onchange={(e) => saveCat(c, { name: e.currentTarget.value })} />
            <input class="cclr" type="color" value={c.color} aria-label="Category color"
              onchange={(e) => saveCat(c, { color: e.currentTarget.value })} />
            <select class="cnat" value={c.nature} aria-label="Category nature"
              onchange={(e) => saveCat(c, { nature: e.currentTarget.value })}>
              <option value="productive">Productive</option>
              <option value="neutral">Neutral</option>
              <option value="distracting">Distracting</option>
            </select>
            <button class="x" onclick={() => dropCat(c)} aria-label={`Delete ${c.name}`}>×</button>
          </li>
        {/each}
      </ul>
      <div class="newcat">
        <input class="cname" bind:value={newName} placeholder="New category…" aria-label="New category name" />
        <input class="cclr" type="color" bind:value={newColor} aria-label="New category color" />
        <select class="cnat" bind:value={newNature} aria-label="New category nature">
          <option value="productive">Productive</option>
          <option value="neutral">Neutral</option>
          <option value="distracting">Distracting</option>
        </select>
        <button class="add" onclick={createCat}>Add</button>
      </div>
      <button class="add ghost done" onclick={() => (managing = false)}>Done</button>
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
  .name { font: inherit; font-size: 14px; font-weight: 600; width: 100%; padding: 2px 4px;
    border: 1px solid transparent; border-radius: var(--radius-sm); background: transparent; color: var(--text); }
  .name:hover, .name:focus { border-color: var(--line); background: var(--bg); outline: none; }
  .exe { color: var(--muted); font-size: 12px; padding-left: 4px; }
  .x { border: none; background: transparent; color: var(--muted);
    font-size: 20px; cursor: pointer; line-height: 1; }
  .who { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
  .who .name, .who .exe { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .launch { border: 1px solid var(--line); background: var(--surface); color: var(--text);
    width: 30px; height: 28px; border-radius: var(--radius-sm); cursor: pointer; font-size: 13px; line-height: 1; }
  .launch:hover { border-color: var(--accent); color: var(--accent); }
  .limit { display: flex; align-items: center; gap: 6px; margin-left: 12px; }
  .cap { width: 52px; font: inherit; font-size: 12px; padding: 4px 6px; text-align: right;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .unit { color: var(--muted); font-size: 11px; }
  .act { font: inherit; font-size: 11px; padding: 4px 8px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--surface); color: var(--muted); cursor: pointer; }
  .act.close { border-color: var(--warn-close); color: var(--warn-close-text); background: var(--warn-close); }
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
  .cat { font: inherit; font-size: 12px; max-width: 130px; flex-shrink: 0; padding: 4px 6px;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .cats { list-style: none; padding: 0; margin: 0 0 14px; display: flex; flex-direction: column; gap: 6px; }
  .cats li, .newcat { display: flex; align-items: center; gap: 8px; }
  .cname { flex: 1; min-width: 0; font: inherit; font-size: 13px; padding: 6px 8px;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .cclr { width: 34px; height: 30px; padding: 0; border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); }
  .cnat { font: inherit; font-size: 12px; padding: 5px 6px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .newcat { margin-bottom: 14px; }
  .err { color: var(--note-overdue); font-size: 12px; margin: 0 0 10px; }
  .done { margin-top: 4px; }
  .meter { height: 5px; width: 100%; margin-top: 6px; border-radius: 3px;
    background: color-mix(in srgb, var(--text) 8%, transparent); overflow: hidden; }
  .mfill { display: block; height: 100%; border-radius: 3px; min-width: 2px; }
  .mfill.over { background: var(--warn-close); }
  .overtxt { color: var(--warn-close); font-size: 11px; margin-top: 3px; }
  .hint { color: var(--muted); font-size: 12px; }
</style>
