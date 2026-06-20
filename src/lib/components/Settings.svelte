<script lang="ts">
  import { onMount } from "svelte";
  import { getSettings, setSetting } from "$lib/api";

  type Toggle = { key: string; label: string; help: string; def: boolean };
  const toggles: Toggle[] = [
    { key: "autostart", label: "Run Korio at startup", help: "Launch automatically when you sign in to Windows.", def: false },
    { key: "start_minimized", label: "Start minimized to tray", help: "Open hidden in the tray instead of showing the window.", def: false },
    { key: "close_to_tray", label: "Close to tray", help: "Keep tracking in the background when you close the window.", def: true },
  ];
  let values = $state<Record<string, boolean>>({});

  onMount(async () => {
    const s = await getSettings();
    const next: Record<string, boolean> = {};
    for (const t of toggles) next[t.key] = s[t.key] !== undefined ? s[t.key] === "true" : t.def;
    values = next;
  });

  async function toggle(key: string) {
    values[key] = !values[key];
    await setSetting(key, values[key] ? "true" : "false");
  }
</script>

<div class="settings">
  <div class="label">System</div>
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
  .row { display: flex; align-items: center; justify-content: space-between; gap: 24px;
    padding: 16px 0; border-bottom: 1px solid var(--line); }
  .name { font-weight: 600; font-size: 14px; }
  .help { color: var(--muted); font-size: 12px; margin-top: 2px; max-width: 440px; }
  .sw { width: 44px; height: 26px; border-radius: 13px; border: 1px solid var(--line); background: var(--bg);
    position: relative; cursor: pointer; flex-shrink: 0; transition: background var(--tick); }
  .sw.on { background: var(--accent); border-color: var(--accent); }
  .knob { position: absolute; top: 2px; left: 2px; width: 20px; height: 20px; border-radius: 50%;
    background: var(--surface); transition: left var(--tick); }
  .sw.on .knob { left: 20px; background: var(--accent-contrast); }
</style>
