<script lang="ts">
  import { dueReminder, dismissReminder, remindLater, formatWhen } from "$lib/reminders";
  import { playSound, getSoundPref } from "$lib/sound";

  let custom = $state(15);
  let showCustom = $state(false);

  // Chime once per reminder as it appears.
  let lastId = $state<number | null>(null);
  $effect(() => {
    const r = $dueReminder;
    if (r && r.id !== lastId) { lastId = r.id; playSound(getSoundPref("timer")); }
    if (!r) lastId = null;
  });

  async function close() { if ($dueReminder) await dismissReminder($dueReminder); showCustom = false; }
  async function later(minutes: number) {
    if ($dueReminder) await remindLater($dueReminder, minutes);
    showCustom = false;
  }
  function onKey(e: KeyboardEvent) { if (e.key === "Escape" && $dueReminder) close(); }
</script>

<svelte:window onkeydown={onKey} />

{#if $dueReminder}
  <div class="overlay" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) close(); }}>
    <div class="dialog" role="alertdialog" aria-label="Reminder">
      <div class="eyebrow">Reminder</div>
      <h3>{$dueReminder.title}</h3>
      <p>Set for {formatWhen($dueReminder.at_ts)}.</p>
      <div class="actions">
        {#if showCustom}
          <label class="cust">In
            <input type="number" min="1" max="1440" bind:value={custom} aria-label="Remind me in minutes" />
            min
          </label>
          <button class="snooze" onclick={() => later(custom)}>Remind then</button>
          <button class="ghost" onclick={() => (showCustom = false)}>Back</button>
        {:else}
          <button class="snooze" onclick={() => later(60)}>Remind in 1 hour</button>
          <button class="ghost" onclick={() => (showCustom = true)}>Choose time…</button>
          <button class="close" onclick={close}>Close reminder</button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,.4); display: flex; align-items: center;
    justify-content: center; z-index: 45; }
  .dialog { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius);
    padding: 24px; width: 400px; max-width: 90vw; }
  .eyebrow { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--accent); }
  h3 { font-family: var(--font-display); font-size: 22px; margin: 6px 0 8px; overflow-wrap: anywhere; }
  p { color: var(--muted); font-size: 13px; margin: 0 0 18px; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; flex-wrap: wrap; align-items: center; }
  .actions button { font: inherit; font-size: 13px; padding: 8px 12px; border-radius: var(--radius-sm);
    border: 1px solid transparent; cursor: pointer; }
  .actions .snooze { background: var(--warn-snooze); color: var(--warn-snooze-text); border-color: var(--line); }
  .actions .close { background: var(--accent); color: var(--accent-contrast); }
  .actions .ghost { background: var(--bg); color: var(--muted); border-color: var(--line); }
  .actions button:hover { filter: brightness(0.95); }
  .cust { font-size: 12px; color: var(--muted); display: flex; align-items: center; gap: 6px; margin-right: auto; }
  .cust input { width: 62px; font: inherit; font-size: 13px; padding: 6px 8px; text-align: right;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
</style>
