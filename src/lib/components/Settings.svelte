<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getSettings, setSetting } from "$lib/api";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { appearance, setMode, setAccent, setTint, type Tint } from "$lib/theme";
  import { hiddenSections, toggleSection } from "$lib/sidebar";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import { exportData, backupDb, restoreDb } from "$lib/api";
  import { hasPin, setPin, clearPin } from "$lib/api";
  import { ensureNotificationPermission } from "$lib/digest";
  import { setCaptureShortcut, browserStatus, setBrowserEnabled, type BrowserStatus } from "$lib/api";
  import { SOUNDS, getSoundPref, setSoundPref, playSound, getLoopPref, setLoopPref, type SoundType, type SoundId } from "$lib/sound";

  let pomoSound = $state(getSoundPref("pomodoro"));
  let timerSound = $state(getSoundPref("timer"));
  function chooseSound(type: SoundType, id: SoundId) {
    setSoundPref(type, id);
    if (type === "pomodoro") pomoSound = id; else timerSound = id;
    playSound(id);
  }

  let limitSound = $state(getSoundPref("limit"));
  let pomoLoop = $state(getLoopPref("pomodoro"));
  let timerLoop = $state(getLoopPref("timer"));
  function chooseLimit(id: SoundId) { setSoundPref("limit", id); limitSound = id; playSound(id); }
  function toggleLoop(t: "pomodoro" | "timer") {
    if (t === "pomodoro") { pomoLoop = !pomoLoop; setLoopPref("pomodoro", pomoLoop); }
    else { timerLoop = !timerLoop; setLoopPref("timer", timerLoop); }
  }

  let dataMsg = $state("");
  let pinSet = $state(false);
  let newPin = $state("");
  let digestEnabled = $state(false);
  let digestTime = $state("18:00");
  let captureEnabled = $state(true);
  let browser = $state<BrowserStatus | null>(null);
  let browserPoll: ReturnType<typeof setInterval> | null = null;

  async function savePin() { if (newPin.length < 4) return; await setPin(newPin); pinSet = true; newPin = ""; }
  async function removePin() { await clearPin(); pinSet = false; }
  async function doExport(format: "csv" | "json") {
    const path = await save({ defaultPath: `korio-export.${format}`,
      filters: [{ name: format.toUpperCase(), extensions: [format] }] });
    if (path) { await exportData(path, format); dataMsg = `Exported to ${path}`; }
  }
  async function doBackup() {
    const path = await save({ defaultPath: "korio-backup.db", filters: [{ name: "Database", extensions: ["db"] }] });
    if (path) { await backupDb(path); dataMsg = `Backed up to ${path}`; }
  }
  async function doRestore() {
    const path = await open({ filters: [{ name: "Database", extensions: ["db"] }], multiple: false });
    if (typeof path === "string") { await restoreDb(path); dataMsg = "Restored. Reopen views to see changes."; }
  }

  const navOptions = [{ id: "stats", label: "Stats" }, { id: "watchlist", label: "Watchlist" }];

  type Toggle = { key: string; label: string; help: string; def: boolean };
  const toggles: Toggle[] = [
    { key: "autostart", label: "Run Korio at startup", help: "Launch automatically when you sign in to Windows.", def: false },
    { key: "start_minimized", label: "Start minimized to tray", help: "Open hidden in the tray instead of showing the window.", def: false },
    { key: "close_to_tray", label: "Close to tray", help: "Keep tracking in the background when you close the window.", def: true },
  ];
  let values = $state<Record<string, boolean>>({});

  const accents = ["#C2410C", "#2F6E4F", "#3A6EA5", "#8A4FB3", "#B23A48", "#1F6F6B", "#B7791F", "#1A1714"];
  const tints: { id: Tint; label: string }[] = [
    { id: "paper", label: "Paper" }, { id: "slate", label: "Slate" },
    { id: "sand", label: "Sand" }, { id: "mono", label: "Mono" },
  ];

  onMount(async () => {
    const s = await getSettings();
    const next: Record<string, boolean> = {};
    for (const t of toggles) next[t.key] = s[t.key] !== undefined ? s[t.key] === "true" : t.def;
    try { next["autostart"] = await isEnabled(); } catch { /* keep stored value */ }
    values = next;
    digestEnabled = s["digest_enabled"] === "true";
    digestTime = s["digest_time"] || "18:00";
    captureEnabled = s["capture_enabled"] !== "false";
    try { pinSet = await hasPin(); } catch { pinSet = false; }
    try { browser = await browserStatus(); } catch { browser = null; }
    browserPoll = setInterval(async () => {
      try { browser = await browserStatus(); } catch { /* keep last */ }
    }, 3000);
  });

  onDestroy(() => { if (browserPoll) clearInterval(browserPoll); });

  async function toggleBrowser() {
    if (!browser) return;
    const next = !browser.enabled;
    browser = { ...browser, enabled: next }; // optimistic, matches other toggles
    await setBrowserEnabled(next);
    try { browser = await browserStatus(); } catch { /* keep optimistic state */ }
  }
  async function copyToken() {
    if (browser?.token) await navigator.clipboard.writeText(browser.token);
  }

  async function toggleCapture() {
    captureEnabled = !captureEnabled;
    await setCaptureShortcut(captureEnabled); // also persists capture_enabled
  }

  async function toggleDigest() {
    const next = !digestEnabled;
    if (next) { await ensureNotificationPermission(); }
    digestEnabled = next;
    await setSetting("digest_enabled", next ? "true" : "false");
  }
  async function onDigestTime(e: Event) {
    digestTime = (e.currentTarget as HTMLInputElement).value || "18:00";
    await setSetting("digest_time", digestTime);
  }

  async function toggle(key: string) {
    values[key] = !values[key];
    if (key === "autostart") {
      try { values[key] ? await enable() : await disable(); } catch { /* ignore */ }
    }
    await setSetting(key, values[key] ? "true" : "false");
  }
</script>

<div class="settings">
  <div class="label">Appearance</div>

  <div class="arow">
    <div class="text"><div class="name">Theme</div><div class="help">Light or dark base.</div></div>
    <div class="seg">
      <button class:on={$appearance.mode === "light"} onclick={() => setMode("light")}>Light</button>
      <button class:on={$appearance.mode === "dark"} onclick={() => setMode("dark")}>Dark</button>
    </div>
  </div>

  <div class="arow">
    <div class="text"><div class="name">Accent</div><div class="help">Used for highlights, the active nav, and key numbers.</div></div>
    <div class="accents">
      {#each accents as a}
        <button class="csw" class:on={$appearance.accent === a} style={`background:${a}`}
          aria-label={`Accent ${a}`} onclick={() => setAccent(a)}></button>
      {/each}
      <input class="picker" type="color" value={$appearance.accent ?? "#C2410C"}
        aria-label="Custom accent color" onchange={(e) => setAccent(e.currentTarget.value)} />
      <button class="reset" onclick={() => setAccent(null)} title="Reset to theme default">Reset</button>
    </div>
  </div>

  <div class="arow">
    <div class="text"><div class="name">Background tint</div><div class="help">Shift the neutral palette.</div></div>
    <div class="tints">
      {#each tints as t}
        <button class:on={$appearance.tint === t.id} onclick={() => setTint(t.id)}>{t.label}</button>
      {/each}
    </div>
  </div>

  <div class="label" style="margin-top:28px">System</div>
  {#each toggles as t}
    <div class="row">
      <div class="text"><div class="name">{t.label}</div><div class="help">{t.help}</div></div>
      <button class="sw" class:on={values[t.key]} role="switch" aria-checked={values[t.key]}
        aria-label={t.label} onclick={() => toggle(t.key)}><span class="knob"></span></button>
    </div>
  {/each}

  <div class="label" style="margin-top:28px">Notifications</div>
  <div class="row">
    <div class="text"><div class="name">End-of-day digest</div>
      <div class="help">A daily summary of your focus time, score, top app, and goals.</div></div>
    <button class="sw" class:on={digestEnabled} role="switch" aria-checked={digestEnabled}
      aria-label="End-of-day digest" onclick={toggleDigest}><span class="knob"></span></button>
  </div>
  {#if digestEnabled}
    <div class="row">
      <div class="text"><div class="name">Digest time</div><div class="help">When the daily summary is shown.</div></div>
      <input class="time" type="time" value={digestTime} onchange={onDigestTime} aria-label="Digest time" />
    </div>
  {/if}

  <div class="row">
    <div class="text"><div class="name">Pomodoro sound</div>
      <div class="help">Plays when a Pomodoro phase ends.</div></div>
    <div class="seg">
      <select aria-label="Pomodoro sound" value={pomoSound}
        onchange={(e) => chooseSound("pomodoro", e.currentTarget.value as SoundId)}>
        {#each SOUNDS as s}<option value={s.id}>{s.label}</option>{/each}
      </select>
      <button class="reset" onclick={() => playSound(pomoSound)} aria-label="Preview pomodoro sound">▶ Preview</button>
    </div>
  </div>
  <div class="row">
    <div class="text"><div class="name">Timer sound</div>
      <div class="help">Plays when a countdown timer finishes.</div></div>
    <div class="seg">
      <select aria-label="Timer sound" value={timerSound}
        onchange={(e) => chooseSound("timer", e.currentTarget.value as SoundId)}>
        {#each SOUNDS as s}<option value={s.id}>{s.label}</option>{/each}
      </select>
      <button class="reset" onclick={() => playSound(timerSound)} aria-label="Preview timer sound">▶ Preview</button>
    </div>
  </div>
  <div class="row">
    <div class="text"><div class="name">Loop Pomodoro sound</div>
      <div class="help">Repeat until you press a Pomodoro button or Stop.</div></div>
    <button class="sw" class:on={pomoLoop} role="switch" aria-checked={pomoLoop}
      aria-label="Loop pomodoro sound" onclick={() => toggleLoop("pomodoro")}><span class="knob"></span></button>
  </div>
  <div class="row">
    <div class="text"><div class="name">Loop Timer sound</div>
      <div class="help">Repeat until you press Start/Reset or Stop.</div></div>
    <button class="sw" class:on={timerLoop} role="switch" aria-checked={timerLoop}
      aria-label="Loop timer sound" onclick={() => toggleLoop("timer")}><span class="knob"></span></button>
  </div>
  <div class="row">
    <div class="text"><div class="name">Limit alert sound</div>
      <div class="help">Plays (looping) when an app or site passes its daily limit, until you respond.</div></div>
    <div class="seg">
      <select aria-label="Limit alert sound" value={limitSound}
        onchange={(e) => chooseLimit(e.currentTarget.value as SoundId)}>
        {#each SOUNDS as s}<option value={s.id}>{s.label}</option>{/each}
      </select>
      <button class="reset" onclick={() => playSound(limitSound)} aria-label="Preview limit sound">▶ Preview</button>
    </div>
  </div>

  <div class="label" style="margin-top:28px">Quick capture</div>
  <div class="row">
    <div class="text"><div class="name">Global hotkey (Ctrl+Alt+K)</div>
      <div class="help">Press Ctrl+Alt+K anywhere to jot a task or note into Korio.</div></div>
    <button class="sw" class:on={captureEnabled} role="switch" aria-checked={captureEnabled}
      aria-label="Global quick-capture" onclick={toggleCapture}><span class="knob"></span></button>
  </div>

  <div class="label" style="margin-top:28px">Browser tracking</div>
  {#if browser}
    <div class="row">
      <div class="text"><div class="name">Track browser sites</div>
        <div class="help">Reports the active tab's domain to Korio via a local-only channel (127.0.0.1). Requires the Korio browser extension.</div></div>
      <button class="sw" class:on={browser.enabled} role="switch" aria-checked={browser.enabled}
        aria-label="Track browser sites" onclick={toggleBrowser}><span class="knob"></span></button>
    </div>
    {#if browser.enabled}
      <div class="row">
        <div class="text"><div class="name">Status</div>
          <div class="help">{#if browser.connected}● Connected{#if browser.domain}{' — '}{browser.domain}{/if}{:else}○ Waiting for extension…{/if}</div></div>
      </div>
      <div class="row">
        <div class="text"><div class="name">Pairing token</div>
          <div class="help">Paste this into the Korio extension's options page (port {browser.port}).</div></div>
        <div class="seg">
          <code class="token-preview">{browser.token.slice(0, 8)}…</code>
          <button onclick={copyToken}>Copy</button>
        </div>
      </div>
    {/if}
  {/if}

  <div class="label" style="margin-top:28px">Sidebar</div>
  {#each navOptions as n}
    <div class="row">
      <div class="text"><div class="name">Show {n.label}</div>
        <div class="help">Display the {n.label} section in the sidebar.</div></div>
      <button class="sw" class:on={!$hiddenSections.includes(n.id)} role="switch"
        aria-checked={!$hiddenSections.includes(n.id)} aria-label={`Show ${n.label}`}
        onclick={() => toggleSection(n.id)}><span class="knob"></span></button>
    </div>
  {/each}

  <div class="label" style="margin-top:28px">Data</div>
  <div class="row">
    <div class="text"><div class="name">Export</div><div class="help">Save your full session log as a file.</div></div>
    <div class="seg"><button onclick={() => doExport("csv")}>CSV</button><button onclick={() => doExport("json")}>JSON</button></div>
  </div>
  <div class="row">
    <div class="text"><div class="name">Backup &amp; restore</div><div class="help">Save or load a complete copy of your database.</div></div>
    <div class="seg"><button onclick={doBackup}>Back up</button><button onclick={doRestore}>Restore…</button></div>
  </div>
  {#if dataMsg}<p class="datamsg">{dataMsg}</p>{/if}

  <div class="label" style="margin-top:28px">Privacy</div>
  <div class="row">
    <div class="text"><div class="name">App lock</div>
      <div class="help">{pinSet ? "A PIN is required to open Korio." : "Require a PIN (4+ digits) to open Korio."}</div></div>
    {#if pinSet}
      <button class="reset" onclick={removePin}>Remove PIN</button>
    {:else}
      <div class="seg">
        <input class="pin" type="password" inputmode="numeric" minlength="4" bind:value={newPin} placeholder="New PIN" aria-label="New PIN" />
        <button onclick={savePin}>Set PIN</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings { max-width: 620px; display: flex; flex-direction: column; gap: 4px; }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 12px; }
  .row, .arow { display: flex; align-items: center; justify-content: space-between; gap: 24px;
    padding: 16px 0; border-bottom: 1px solid var(--line); }
  .name { font-weight: 600; font-size: 14px; }
  .help { color: var(--muted); font-size: 12px; margin-top: 2px; max-width: 440px; }

  /* system toggle switch */
  .sw { width: 44px; height: 26px; border-radius: 13px; border: 1px solid var(--line); background: var(--bg);
    position: relative; cursor: pointer; flex-shrink: 0; transition: background var(--tick); }
  .sw.on { background: var(--accent); border-color: var(--accent); }
  .knob { position: absolute; top: 2px; left: 2px; width: 20px; height: 20px; border-radius: 50%;
    background: var(--surface); transition: left var(--tick); }
  .sw.on .knob { left: 20px; background: var(--accent-contrast); }

  /* appearance controls */
  .seg, .tints { display: flex; gap: 4px; }
  .seg button, .tints button, .reset { font: inherit; font-size: 13px; padding: 7px 12px;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--surface);
    color: var(--muted); cursor: pointer; }
  .seg button.on, .tints button.on { background: var(--accent); color: var(--accent-contrast); border-color: var(--accent); }
  .seg button:hover, .tints button:hover, .reset:hover { color: var(--text); }
  .accents { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; justify-content: flex-end; }
  .csw { width: 24px; height: 24px; border-radius: 7px; border: 2px solid transparent; cursor: pointer; padding: 0; }
  .csw.on { border-color: var(--text); }
  .picker { width: 30px; height: 30px; padding: 0; border: 1px solid var(--line); border-radius: 7px; background: none; cursor: pointer; }
  .datamsg { color: var(--muted); font-size: 12px; padding-top: 10px; }
  .pin { width: 90px; font: inherit; font-size: 13px; padding: 6px 8px; border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .time { font: inherit; font-size: 13px; padding: 6px 8px; border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .token-preview { font-family: monospace; font-size: 13px; color: var(--muted); padding: 6px 8px; background: var(--bg); border: 1px solid var(--line); border-radius: var(--radius-sm); }
  .seg select { font: inherit; font-size: 13px; padding: 6px 8px; border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); cursor: pointer; }
</style>
