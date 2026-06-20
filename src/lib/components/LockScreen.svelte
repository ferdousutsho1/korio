<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { hasPin, verifyPin } from "$lib/api";

  let locked = $state(false);
  let pin = $state("");
  let error = $state(false);

  onMount(async () => {
    if (!browser) return;
    try { locked = await hasPin(); } catch { locked = false; }
  });

  async function submit(e: Event) {
    e.preventDefault();
    try {
      if (await verifyPin(pin)) { locked = false; pin = ""; error = false; }
      else { error = true; pin = ""; }
    } catch { locked = false; }
  }
</script>

{#if locked}
  <div class="lock">
    <form class="box" onsubmit={submit}>
      <div class="brand">Korio</div>
      <p>Enter your PIN to unlock.</p>
      <!-- svelte-ignore a11y_autofocus -->
      <input type="password" inputmode="numeric" bind:value={pin} class:err={error} aria-label="PIN" autofocus />
      <button type="submit">Unlock</button>
      {#if error}<div class="msg">Incorrect PIN.</div>{/if}
    </form>
  </div>
{/if}

<style>
  .lock { position: fixed; inset: 0; z-index: 100; background: var(--bg);
    display: flex; align-items: center; justify-content: center; }
  .box { display: flex; flex-direction: column; gap: 12px; width: 280px; text-align: center;
    background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 32px; }
  .brand { font-family: var(--font-display); font-size: 28px; color: var(--text); }
  p { color: var(--muted); font-size: 13px; margin: 0; }
  input { font: inherit; font-size: 18px; letter-spacing: 6px; text-align: center; padding: 10px;
    border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg); color: var(--text); }
  input.err { border-color: #B23A48; }
  button { font: inherit; padding: 10px; border-radius: var(--radius-sm); border: none;
    background: var(--accent); color: var(--accent-contrast); cursor: pointer; }
  .msg { color: #B23A48; font-size: 12px; }
</style>
