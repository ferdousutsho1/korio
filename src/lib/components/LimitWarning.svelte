<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { browser } from "$app/environment";
  import { listen } from "@tauri-apps/api/event";
  import { snoozeLimit, ignoreLimit, forceClose, snoozeSiteLimit, ignoreSiteLimit, blockSite, clearAlertTopmost, type LimitEvent } from "$lib/api";
  import { startLoop, stopAllSounds, getSoundPref } from "$lib/sound";
  import { formatDuration } from "$lib/format";

  let current = $state<LimitEvent | null>(null);
  let closedToast = $state<string | null>(null);
  let countdown = $state<number | null>(null);
  let countdownTimer: ReturnType<typeof setInterval> | null = null;
  function stopCountdown() {
    if (countdownTimer) { clearInterval(countdownTimer); countdownTimer = null; }
    countdown = null;
  }

  let stopLoop: (() => void) | null = null;
  function endAlert() {
    stopCountdown();
    stopLoop?.(); stopLoop = null;
    clearAlertTopmost().catch(() => {});
  }
  onDestroy(() => { stopAllSounds(); stopCountdown(); });

  onMount(() => {
    if (!browser) return;
    const unlistens: Array<() => void> = [];
    listen<LimitEvent>("limit-reached", (e) => {
      current = e.payload;
      stopLoop?.();
      stopLoop = startLoop(getSoundPref("limit"));
      stopCountdown();
      if (e.payload.auto_close) {
        countdown = 5;
        countdownTimer = setInterval(() => {
          if (countdown !== null) countdown -= 1;
          if (countdown !== null && countdown <= 0) { stopCountdown(); close(); }
        }, 1000);
      }
    }).then((u) => unlistens.push(u));
    listen<string>("limit-closed", (e) => {
      closedToast = e.payload;
      setTimeout(() => { if (closedToast === e.payload) closedToast = null; }, 4000);
    }).then((u) => unlistens.push(u));
    return () => unlistens.forEach((u) => u());
  });

  async function snooze() {
    if (!current) return;
    if (current.kind === "site") await snoozeSiteLimit(current.exe, 10);
    else await snoozeLimit(current.exe, 10);
    endAlert(); current = null;
  }
  async function ignore() {
    if (!current) return;
    if (current.kind === "site") await ignoreSiteLimit(current.exe);
    else await ignoreLimit(current.exe);
    endAlert(); current = null;
  }
  async function close() {
    if (!current) return;
    if (current.kind === "site") await blockSite(current.exe);
    else await forceClose(current.exe);
    endAlert(); current = null;
  }

  function onKey(e: KeyboardEvent) { if (e.key === "Escape" && current) ignore(); }
</script>

<svelte:window onkeydown={onKey} />

{#if current}
  <div class="overlay" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) ignore(); }}>
    <div class="dialog" role="alertdialog" aria-label="Daily limit reached">
      <div class="eyebrow">Daily limit reached</div>
      <h3>{current.display_name}</h3>
      <p>You've used <strong>{formatDuration(current.today_seconds)}</strong> today — past your
        {formatDuration(current.cap_seconds)} limit.{#if countdown !== null}
        <br /><strong class="cd">Closing in {countdown}s…</strong>{/if}</p>
      <div class="actions">
        <button class="snooze" onclick={snooze}>Snooze 10 min</button>
        <button class="ignore" onclick={ignore}>Ignore today</button>
        <button class="close" onclick={close}>{current.kind === "site" ? "Close tab" : "Close app"}</button>
      </div>
    </div>
  </div>
{/if}

{#if closedToast}
  <div class="toast">Closed <strong>{closedToast}</strong> — daily limit reached.</div>
{/if}

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,.4); display: flex; align-items: center;
    justify-content: center; z-index: 50; }
  .dialog { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius);
    padding: 24px; width: 380px; max-width: 90vw; }
  .eyebrow { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--accent); }
  h3 { font-family: var(--font-display); font-size: 22px; margin: 6px 0 8px; }
  p { color: var(--muted); font-size: 14px; margin: 0 0 18px; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; flex-wrap: wrap; }
  .actions button { font: inherit; font-size: 13px; padding: 8px 12px; border-radius: var(--radius-sm);
    border: 1px solid transparent; cursor: pointer; }
  .actions .snooze { background: var(--warn-snooze); color: var(--warn-snooze-text); border-color: var(--line); }
  .actions .ignore { background: var(--warn-ignore); color: var(--warn-ignore-text); }
  .actions .close { background: var(--warn-close); color: var(--warn-close-text); }
  .actions button:hover { filter: brightness(0.95); }
  .cd { color: var(--warn-close); }
  .toast { position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%); z-index: 50;
    background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius-sm);
    padding: 10px 16px; font-size: 13px; color: var(--text); box-shadow: 0 6px 24px rgba(0,0,0,.18); }
</style>
