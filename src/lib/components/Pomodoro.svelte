<script lang="ts">
  import { nowMs } from "$lib/tools";
  import { pomodoro, config, setConfig, pomoStartPause, pomoReset, pomoSkip, pomoRemaining, type PomodoroConfig } from "$lib/pomodoro";
  import { formatClock } from "$lib/format";

  let remaining = $derived(pomoRemaining($pomodoro, $nowMs));
  let label = $state("");
  let prevTransitions = -1; // -1 = not yet observed (don't chime on mount)

  const phaseName = (p: string) => p === "focus" ? "Focus" : p === "short" ? "Short break" : "Long break";

  $effect(() => {
    const t = $pomodoro.transitions;
    if (prevTransitions !== -1 && t !== prevTransitions) chime();
    prevTransitions = t;
  });

  function chime() {
    try {
      const Ctx = window.AudioContext || (window as any).webkitAudioContext;
      const ctx = new Ctx();
      [660, 880].forEach((f, i) => {
        const o = ctx.createOscillator(); const g = ctx.createGain();
        o.frequency.value = f; o.connect(g); g.connect(ctx.destination);
        const t0 = ctx.currentTime + i * 0.18;
        g.gain.setValueAtTime(0.0001, t0);
        g.gain.exponentialRampToValueAtTime(0.25, t0 + 0.02);
        g.gain.exponentialRampToValueAtTime(0.0001, t0 + 0.16);
        o.start(t0); o.stop(t0 + 0.18);
      });
    } catch { /* no audio */ }
  }

  function updateCfg(patch: Partial<PomodoroConfig>) { setConfig({ ...$config, ...patch }); }
</script>

<div class="pomo" data-phase={$pomodoro.phase}>
  <div class="badge">{phaseName($pomodoro.phase)}</div>
  <div class="time">{formatClock(Math.ceil(remaining / 1000))}</div>

  {#if $pomodoro.phase === "focus"}
    <input class="focuslabel" placeholder="What are you focusing on? (optional)" bind:value={label} aria-label="Focus label" />
  {/if}

  <div class="btns">
    <button class="primary" onclick={pomoStartPause}>{$pomodoro.running ? "Pause" : "Start"}</button>
    <button onclick={pomoSkip}>Skip</button>
    <button onclick={pomoReset}>Reset</button>
  </div>

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
</style>
