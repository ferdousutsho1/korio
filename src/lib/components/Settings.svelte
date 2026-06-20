<script lang="ts">
  import { onMount } from "svelte";
  import { getSettings, setSetting } from "$lib/api";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { appearance, setMode, setAccent, setTint, type Tint } from "$lib/theme";

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
  });

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
</style>
