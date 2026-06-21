<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { closeWidget, setWidgetAlwaysOnTop, saveWidgetBounds, type WidgetKind } from "$lib/api";
  import { contrastColor } from "$lib/theme";
  import { nowMs, stopwatch, swStartStop, swLap, swReset, swElapsed,
           timer, timerStartPause, timerReset, timerRemaining } from "$lib/tools";
  import { pomodoro, config, pomoStartPause, pomoSkip, pomoRemaining } from "$lib/pomodoro";
  import { formatStopwatch, formatClock } from "$lib/format";

  let { kind }: { kind: WidgetKind } = $props();

  const ACCENTS = ["#C2410C", "#B23A48", "#7A6F5C", "#3A6EA5", "#2F6E4F", "#8A4FB3"];
  let opacity = $state(1);
  let accent = $state<string | null>(null);
  let aot = $state(true);
  let showCfg = $state(false);
  let clockZones = $state<string[]>([]);

  const KEY = `korio.widget.${kind}`;
  function persistPrefs() {
    if (browser) localStorage.setItem(KEY, JSON.stringify({ opacity, accent, aot }));
  }

  // live tickers
  let swMs = $derived(swElapsed($stopwatch, $nowMs));
  let tmMs = $derived(timerRemaining($timer, $nowMs));
  let pmMs = $derived(pomoRemaining($pomodoro, $nowMs));
  const phaseLabel = (p: string) => (p === "focus" ? "Focus" : p === "short" ? "Short break" : "Long break");

  function timeIn(zone: string, _now: number) {
    try { return new Intl.DateTimeFormat(undefined, { timeZone: zone, hour: "2-digit", minute: "2-digit" }).format(new Date()); }
    catch { return "—"; }
  }

  function toggleAot() { aot = !aot; persistPrefs(); setWidgetAlwaysOnTop(kind, aot); }
  function pickAccent(a: string | null) { accent = a; persistPrefs(); }
  function onOpacity(e: Event) { opacity = Number((e.currentTarget as HTMLInputElement).value); persistPrefs(); }

  onMount(() => {
    if (browser) {
      try {
        const raw = JSON.parse(localStorage.getItem(KEY) || "{}");
        if (typeof raw.opacity === "number") opacity = raw.opacity;
        if (typeof raw.accent === "string" || raw.accent === null) accent = raw.accent;
        if (typeof raw.aot === "boolean") aot = raw.aot;
      } catch { /* defaults */ }
      try {
        const v = JSON.parse(localStorage.getItem("korio.worldclock") || "null");
        clockZones = Array.isArray(v) && v.length ? v : [Intl.DateTimeFormat().resolvedOptions().timeZone, "UTC"];
      } catch { clockZones = [Intl.DateTimeFormat().resolvedOptions().timeZone, "UTC"]; }
      // transparent window: let the desktop show through around the card
      document.documentElement.style.background = "transparent";
      document.body.style.background = "transparent";
    }
    let t: ReturnType<typeof setTimeout>;
    let unMoved: (() => void) | undefined;
    let unResized: (() => void) | undefined;
    (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        const save = async () => {
          const scale = await win.scaleFactor();
          const pos = (await win.outerPosition()).toLogical(scale);
          const size = (await win.innerSize()).toLogical(scale);
          saveWidgetBounds(kind, pos.x, pos.y, size.width, size.height);
        };
        const debounced = () => { clearTimeout(t); t = setTimeout(save, 400); };
        unMoved = await win.onMoved(debounced);
        unResized = await win.onResized(debounced);
      } catch { /* not in Tauri */ }
    })();
    return () => { clearTimeout(t); unMoved?.(); unResized?.(); };
  });
</script>

<div class="widget" style="opacity:{opacity};{accent ? `--accent:${accent};--accent-contrast:${contrastColor(accent)}` : ''}">
  <div class="bar" data-tauri-drag-region>
    <span class="title" data-tauri-drag-region>{kind}</span>
    <button class="icon" title="Settings" aria-label="Widget settings" onclick={() => (showCfg = !showCfg)}>⚙</button>
    <button class="icon" title="Close" aria-label="Close widget" onclick={() => closeWidget(kind)}>×</button>
  </div>

  {#if showCfg}
    <div class="cfg">
      <label class="row"><span>Opacity</span>
        <input type="range" min="0.3" max="1" step="0.05" value={opacity} oninput={onOpacity} /></label>
      <div class="row"><span>Accent</span>
        <div class="swatches">
          <button class="sw" class:on={accent === null} style="background:var(--accent)" title="Theme default" aria-label="Theme accent" onclick={() => pickAccent(null)}></button>
          {#each ACCENTS as a}
            <button class="sw" class:on={accent === a} style="background:{a}" title={a} aria-label={a} onclick={() => pickAccent(a)}></button>
          {/each}
        </div>
      </div>
      <label class="row"><span>Always on top</span>
        <input type="checkbox" checked={aot} onchange={toggleAot} /></label>
    </div>
  {/if}

  <div class="body">
    {#if kind === "stopwatch"}
      <div class="time">{formatStopwatch(swMs)}</div>
      <div class="btns">
        <button class="primary" onclick={swStartStop}>{$stopwatch.running ? "Stop" : "Start"}</button>
        <button onclick={() => swLap(swMs)} disabled={!$stopwatch.running}>Lap</button>
        <button onclick={swReset} disabled={$stopwatch.running || swMs === 0}>Reset</button>
      </div>
    {:else if kind === "timer"}
      <div class="time" class:alarm={$timer.done}>{formatClock(Math.ceil(tmMs / 1000))}</div>
      <div class="btns">
        <button class="primary" onclick={timerStartPause} disabled={tmMs <= 0 && !$timer.running}>{$timer.running ? "Pause" : "Start"}</button>
        <button onclick={timerReset} disabled={$timer.durationMs === 0}>Reset</button>
      </div>
    {:else if kind === "pomodoro"}
      <div class="phase">{phaseLabel($pomodoro.phase)}</div>
      <div class="time">{formatClock(Math.ceil(pmMs / 1000))}</div>
      <div class="btns">
        <button class="primary" onclick={pomoStartPause}>{$pomodoro.running ? "Pause" : "Start"}</button>
        <button onclick={pomoSkip}>Skip</button>
      </div>
      <div class="sub">{$pomodoro.completedFocus} focus today · cfg {$config.focusMin}m</div>
    {:else}
      <ul class="clocks">
        {#each clockZones as z}
          <li><span class="z">{z.split("/").pop()?.replace(/_/g, " ")}</span>
            <span class="zt">{timeIn(z, $nowMs)}</span></li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  :global(html), :global(body) { background: transparent !important; }
  .widget { font-family: var(--font-body); color: var(--text); height: 100vh; box-sizing: border-box;
    display: flex; flex-direction: column; background: var(--surface);
    border: 1px solid var(--line); border-radius: var(--radius); overflow: hidden; }
  .bar { display: flex; align-items: center; gap: 6px; padding: 4px 6px 4px 10px; cursor: move;
    background: color-mix(in srgb, var(--text) 5%, transparent); }
  .title { flex: 1; font-size: 11px; text-transform: capitalize; color: var(--muted); }
  .icon { border: none; background: none; color: var(--muted); font-size: 15px; line-height: 1; cursor: pointer; padding: 2px 4px; }
  .icon:hover { color: var(--text); }
  .cfg { display: flex; flex-direction: column; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--line); font-size: 11px; }
  .cfg .row { display: flex; align-items: center; justify-content: space-between; gap: 8px; color: var(--muted); }
  .swatches { display: flex; gap: 4px; }
  .sw { width: 14px; height: 14px; border-radius: 50%; border: 1px solid color-mix(in srgb, var(--text) 20%, transparent); cursor: pointer; padding: 0; }
  .sw.on { outline: 2px solid var(--text); outline-offset: 1px; }
  .body { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; padding: 10px; }
  .time { font-family: var(--font-display); font-size: 34px; font-variant-numeric: tabular-nums; line-height: 1; }
  .time.alarm { color: var(--accent); }
  .phase { font-size: 12px; color: var(--muted); }
  .sub { font-size: 10px; color: var(--muted); }
  .btns { display: flex; gap: 6px; }
  .btns button { font: inherit; font-size: 12px; padding: 6px 12px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--bg); color: var(--text); cursor: pointer; }
  .btns button:disabled { opacity: .4; cursor: default; }
  .primary { background: var(--accent) !important; color: var(--accent-contrast) !important; border-color: var(--accent) !important; }
  .clocks { list-style: none; margin: 0; padding: 0; width: 100%; display: flex; flex-direction: column; gap: 6px; }
  .clocks li { display: flex; justify-content: space-between; font-size: 13px; border-bottom: 1px solid var(--line); padding-bottom: 4px; }
  .clocks .zt { font-variant-numeric: tabular-nums; }
</style>
