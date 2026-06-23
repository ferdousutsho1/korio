<script lang="ts">
  import { nowMs } from "$lib/tools";
  import { pomodoro, config, setConfig, pomoStartPause, pomoReset, pomoSkip, pomoRemaining, type PomodoroConfig } from "$lib/pomodoro";
  import { formatClock } from "$lib/format";
  import { playSound, getSoundPref, getLoopPref, startLoop, stopAllSounds } from "$lib/sound";
  import { onDestroy } from "svelte";

  let remaining = $derived(pomoRemaining($pomodoro, $nowMs));
  let label = $state("");
  let prevTransitions = -1; // -1 = not yet observed (don't chime on mount)

  const phaseName = (p: string) => p === "focus" ? "Focus" : p === "short" ? "Short break" : "Long break";

  $effect(() => {
    const t = $pomodoro.transitions;
    if (prevTransitions !== -1 && t !== prevTransitions) chime();
    prevTransitions = t;
  });

  let stopLoop = $state<(() => void) | null>(null);
  function stopSound() { stopLoop?.(); stopLoop = null; }
  function chime() {
    stopSound();
    if (getLoopPref("pomodoro")) stopLoop = startLoop(getSoundPref("pomodoro"));
    else playSound(getSoundPref("pomodoro"));
  }
  onDestroy(() => stopAllSounds());

  function updateCfg(patch: Partial<PomodoroConfig>) { setConfig({ ...$config, ...patch }); }
</script>

<div class="pomo" data-phase={$pomodoro.phase}>
  <div class="badge">{phaseName($pomodoro.phase)}</div>
  <div class="time">{formatClock(Math.ceil(remaining / 1000))}</div>

  {#if $pomodoro.phase === "focus"}
    <input class="focuslabel" placeholder="What are you focusing on? (optional)" bind:value={label} aria-label="Focus label" />
  {/if}

  <div class="btns">
    <button class="primary" onclick={() => { stopSound(); pomoStartPause(); }}>{$pomodoro.running ? "Pause" : "Start"}</button>
    <button onclick={() => { stopSound(); pomoSkip(); }}>Skip</button>
    <button onclick={() => { stopSound(); pomoReset(); }}>Reset</button>
  </div>
  {#if stopLoop}<button class="stopsnd" onclick={stopSound}>🔇 Stop sound</button>{/if}

  <div class="count">🍅 {$pomodoro.completedFocus} focus {$pomodoro.completedFocus === 1 ? "session" : "sessions"} today</div>

  <div class="cfg">
    <label>Focus <input type="number" min="1" max="180" value={$config.focusMin} onchange={(e) => updateCfg({ focusMin: +e.currentTarget.value })} /></label>
    <label>Short <input type="number" min="1" max="60" value={$config.shortMin} onchange={(e) => updateCfg({ shortMin: +e.currentTarget.value })} /></label>
    <label>Long <input type="number" min="1" max="60" value={$config.longMin} onchange={(e) => updateCfg({ longMin: +e.currentTarget.value })} /></label>
    <label>Long every <input type="number" min="1" max="12" value={$config.longEvery} onchange={(e) => updateCfg({ longEvery: +e.currentTarget.value })} /></label>
  </div>
</div>

<style>
  .pomo { display: flex; flex-direction: column; gap: 16px; align-items: center; }
  .badge { font-size: 11px; letter-spacing: 2px; text-transform: uppercase; padding: 5px 12px;
    border-radius: 20px; background: var(--chip); color: var(--muted); }
  .pomo[data-phase="focus"] .badge { background: var(--accent); color: var(--accent-contrast); }
  .time { font-family: var(--font-display); font-size: 64px; font-variant-numeric: tabular-nums; color: var(--text); }
  .focuslabel { width: 100%; max-width: 360px; font: inherit; font-size: 13px; text-align: center;
    padding: 8px 10px; border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .btns { display: flex; gap: 8px; }
  .btns button { font: inherit; font-size: 14px; padding: 9px 18px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--bg); color: var(--text); cursor: pointer; }
  .primary { background: var(--accent) !important; color: var(--accent-contrast) !important; border-color: var(--accent) !important; }
  .count { color: var(--muted); font-size: 13px; }
  .cfg { display: flex; gap: 14px; flex-wrap: wrap; justify-content: center; border-top: 1px solid var(--line);
    padding-top: 16px; width: 100%; }
  .cfg label { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--muted); }
  .cfg input { width: 54px; font: inherit; padding: 5px 7px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--text); text-align: right; }
  .stopsnd { margin-left: 10px; font: inherit; font-size: 12px; padding: 3px 10px; border-radius: var(--radius-sm);
    border: 1px solid var(--accent); background: var(--surface); color: var(--accent); cursor: pointer; }
</style>
