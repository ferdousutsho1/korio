<script lang="ts">
  import { verifyPin } from "$lib/api";
  let { title = "Enter your PIN", hint = "", onOk, onCancel }:
    { title?: string; hint?: string; onOk: () => void; onCancel: () => void } = $props();

  let pin = $state("");
  let error = $state(false);
  let checking = $state(false);

  async function submit(e: Event) {
    e.preventDefault();
    if (checking) return;
    checking = true;
    try {
      if (await verifyPin(pin)) { pin = ""; error = false; onOk(); }
      else { error = true; pin = ""; }
    } catch { error = true; }
    checking = false;
  }
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label={title} tabindex="-1"
  onclick={(e) => { if (e.target === e.currentTarget) onCancel(); }}
  onkeydown={(e) => { if (e.key === "Escape") onCancel(); }}>
  <form class="box" onsubmit={submit}>
    <div class="eyebrow">Locked</div>
    <h4>{title}</h4>
    {#if hint}<p>{hint}</p>{/if}
    <!-- svelte-ignore a11y_autofocus -->
    <input type="password" inputmode="numeric" bind:value={pin} class:err={error} aria-label="PIN" autofocus />
    {#if error}<div class="msg">Incorrect PIN.</div>{/if}
    <div class="btns">
      <button type="button" class="ghost" onclick={onCancel}>Cancel</button>
      <button type="submit" disabled={checking || pin.length === 0}>Unlock</button>
    </div>
  </form>
</div>

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,.5); z-index: 60;
    display: flex; align-items: center; justify-content: center; }
  .box { display: flex; flex-direction: column; gap: 10px; width: 300px; text-align: center;
    background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 26px; }
  .eyebrow { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--accent); }
  h4 { font-family: var(--font-display); font-size: 18px; margin: 0; }
  p { color: var(--muted); font-size: 12px; margin: 0; }
  input { font: inherit; font-size: 18px; letter-spacing: 6px; text-align: center; padding: 10px;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  input.err { border-color: var(--warn-close); }
  .msg { color: var(--warn-close); font-size: 12px; }
  .btns { display: flex; gap: 8px; }
  .btns button { flex: 1; font: inherit; font-size: 13px; padding: 9px; border-radius: var(--radius-sm);
    border: none; background: var(--accent); color: var(--accent-contrast); cursor: pointer; }
  .btns button:disabled { opacity: .5; cursor: default; }
  .btns .ghost { background: var(--bg); color: var(--muted); border: 1px solid var(--line); }
</style>
