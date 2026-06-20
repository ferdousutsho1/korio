<script lang="ts">
  import { nowMs, timer, timerSet, timerStartPause, timerReset, timerRemaining, timerTick } from "$lib/tools";
  import { formatClock } from "$lib/format";

  const presets = [1, 5, 10, 25];
  let mins = $state(5);
  let remaining = $derived(timerRemaining($timer, $nowMs));
  let prevDone = false;

  $effect(() => {
    timerTick($nowMs);
    if ($timer.done && !prevDone) beep();
    prevDone = $timer.done;
  });

  function applyPreset(m: number) { mins = m; timerSet(m * 60_000); }
  function setCustom() { timerSet(Math.max(0, Math.round(mins)) * 60_000); }

  function beep() {
    try {
      const Ctx = window.AudioContext || (window as any).webkitAudioContext;
      const ctx = new Ctx();
      const o = ctx.createOscillator(); const g = ctx.createGain();
      o.frequency.value = 880; o.connect(g); g.connect(ctx.destination);
      g.gain.setValueAtTime(0.001, ctx.currentTime);
      g.gain.exponentialRampToValueAtTime(0.3, ctx.currentTime + 0.02);
      g.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.6);
      o.start(); o.stop(ctx.currentTime + 0.6);
    } catch { /* audio not available */ }
  }
</script>

<div class="tm" class:done={$timer.done}>
  <div class="time">{formatClock(Math.ceil(remaining / 1000))}</div>
  <div class="presets">
    {#each presets as p}<button class:on={mins === p} onclick={() => applyPreset(p)}>{p}m</button>{/each}
    <span class="custom"><input type="number" min="0" bind:value={mins} onchange={setCustom} aria-label="Minutes" /><span>min</span></span>
  </div>
  <div class="btns">
    <button class="primary" onclick={timerStartPause} disabled={remaining <= 0 && !$timer.running}>
      {$timer.running ? "Pause" : "Start"}
    </button>
    <button onclick={timerReset} disabled={$timer.durationMs === 0}>Reset</button>
  </div>
  {#if $timer.done}<div class="alarm">⏰ Time's up</div>{/if}
</div>

<style>
  .tm { display: flex; flex-direction: column; gap: 18px; align-items: center; }
  .time { font-family: var(--font-display); font-size: 56px; font-variant-numeric: tabular-nums; color: var(--text); }
  .tm.done .time { color: var(--accent); }
  .presets { display: flex; align-items: center; gap: 6px; }
  .presets button { font: inherit; font-size: 13px; padding: 7px 12px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--muted); cursor: pointer; }
  .presets button.on { background: var(--accent); color: var(--accent-contrast); border-color: var(--accent); }
  .custom { display: flex; align-items: center; gap: 6px; color: var(--muted); font-size: 12px; }
  .custom input { width: 64px; font: inherit; padding: 6px 8px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  .btns { display: flex; gap: 8px; }
  .btns button { font: inherit; font-size: 14px; padding: 9px 18px; border-radius: var(--radius-sm);
    border: 1px solid var(--line); background: var(--bg); color: var(--text); cursor: pointer; }
  .btns button:disabled { opacity: .4; cursor: default; }
  .primary { background: var(--accent) !important; color: var(--accent-contrast) !important; border-color: var(--accent) !important; }
  .alarm { color: var(--accent); font-weight: 600; }
</style>
