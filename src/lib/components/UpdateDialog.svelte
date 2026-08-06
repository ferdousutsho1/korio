<script lang="ts">
  import { updateStatus, updateDialogOpen, currentVersion, installUpdate,
    skipVersion, remindLater, checkForUpdates, progressFraction } from "$lib/updates";

  let s = $derived($updateStatus);
  let busy = $derived(s.state === "downloading" || s.state === "ready");

  function fmtBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
    return `${(n / 1048576).toFixed(1)} MB`;
  }
  function fmtDate(d: string | null): string {
    if (!d) return "";
    // The updater sends RFC3339-ish dates like "2026-08-06 11:02:39.0 +00:00:00".
    const parsed = new Date(d.replace(/\.\d+\s/, " ").replace(/\s\+(\d{2}):(\d{2}):\d{2}$/, "+$1:$2"));
    return Number.isNaN(parsed.getTime())
      ? "" : parsed.toLocaleDateString(undefined, { month: "long", day: "numeric", year: "numeric" });
  }

  /** Never let a download be interrupted by a stray click or Esc. */
  function dismiss() { if (!busy) updateDialogOpen.set(false); }
  function onKey(e: KeyboardEvent) { if (e.key === "Escape" && $updateDialogOpen) dismiss(); }

  let pct = $derived(s.state === "downloading" ? progressFraction(s.downloaded, s.total) : null);
</script>

<svelte:window onkeydown={onKey} />

{#if $updateDialogOpen}
  <div class="overlay" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) dismiss(); }}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Software update">
      {#if s.state === "available" || s.state === "downloading" || s.state === "ready"}
        <div class="eyebrow">Update available</div>
        <h3>Korio {s.version}</h3>
        <p class="meta">
          You're on {$currentVersion || "an older version"}{#if s.state === "available" && fmtDate(s.date)}{' · released '}{fmtDate(s.date)}{/if}
        </p>

        {#if s.state === "available"}
          {#if s.notes.length}
            <div class="notes">
              <div class="nlabel">What's new</div>
              <ul>{#each s.notes as n}<li>{n}</li>{/each}</ul>
            </div>
          {:else}
            <p class="meta">No release notes were published for this version.</p>
          {/if}
          <div class="actions">
            <button class="ghost" onclick={() => skipVersion(s.version)}>Skip this version</button>
            <button class="ghost" onclick={() => remindLater(24)}>Remind me later</button>
            <button class="primary" onclick={installUpdate}>Update &amp; restart</button>
          </div>
        {:else if s.state === "downloading"}
          <div class="progress">
            <div class="track"><div class="fill" class:indet={pct === null}
              style={pct === null ? "" : `width:${pct * 100}%`}></div></div>
            <div class="ptxt">
              {#if s.total}Downloading… {fmtBytes(s.downloaded)} of {fmtBytes(s.total)}
              {:else}Downloading… {fmtBytes(s.downloaded)}{/if}
            </div>
          </div>
          <p class="meta">Korio will close and reopen on its own when this finishes.</p>
        {:else}
          <div class="progress">
            <div class="track"><div class="fill" style="width:100%"></div></div>
            <div class="ptxt">Installing and restarting…</div>
          </div>
        {/if}

      {:else if s.state === "uptodate"}
        <div class="eyebrow">Up to date</div>
        <h3>Korio {$currentVersion}</h3>
        <p class="meta">You're running the latest version.</p>
        <div class="actions"><button class="primary" onclick={dismiss}>Close</button></div>

      {:else if s.state === "error"}
        <div class="eyebrow err">Couldn't check for updates</div>
        <h3>Something went wrong</h3>
        <p class="meta code">{s.message}</p>
        <p class="meta">Check your connection and try again, or download the latest build from GitHub.</p>
        <div class="actions">
          <button class="ghost" onclick={dismiss}>Close</button>
          <button class="primary" onclick={() => checkForUpdates(true)}>Try again</button>
        </div>

      {:else}
        <div class="eyebrow">Checking</div>
        <h3>Looking for updates…</h3>
        <div class="actions"><button class="ghost" onclick={dismiss}>Cancel</button></div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,.4); display: flex; align-items: center;
    justify-content: center; z-index: 55; }
  .dialog { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius);
    padding: 24px; width: 440px; max-width: 90vw; }
  .eyebrow { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--accent); }
  .eyebrow.err { color: var(--warn-close); }
  h3 { font-family: var(--font-display); font-size: 22px; margin: 6px 0 8px; }
  .meta { color: var(--muted); font-size: 13px; margin: 0 0 14px; }
  .meta.code { font-family: monospace; font-size: 11px; overflow-wrap: anywhere;
    background: var(--bg); border: 1px solid var(--line); border-radius: var(--radius-sm); padding: 8px; }

  .notes { border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--bg);
    padding: 12px 14px; margin-bottom: 18px; max-height: 220px; overflow: auto; }
  .nlabel { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 8px; }
  .notes ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
  .notes li { font-size: 12.5px; line-height: 1.5; overflow-wrap: anywhere; }

  .progress { margin: 4px 0 14px; }
  .track { height: 8px; border-radius: 4px; overflow: hidden;
    background: color-mix(in srgb, var(--text) 8%, transparent); }
  .fill { height: 100%; background: var(--accent); border-radius: 4px; transition: width .2s linear; }
  .fill.indet { width: 40%; animation: slide 1.2s ease-in-out infinite; }
  @keyframes slide { 0% { margin-left: -40%; } 100% { margin-left: 100%; } }
  .ptxt { font-size: 12px; color: var(--muted); margin-top: 8px; font-variant-numeric: tabular-nums; }
  @media (prefers-reduced-motion: reduce) { .fill.indet { animation: none; width: 100%; } }

  .actions { display: flex; gap: 8px; justify-content: flex-end; flex-wrap: wrap; }
  .actions button { font: inherit; font-size: 13px; padding: 8px 12px; border-radius: var(--radius-sm);
    border: 1px solid transparent; cursor: pointer; }
  .actions .primary { background: var(--accent); color: var(--accent-contrast); }
  .actions .ghost { background: var(--bg); color: var(--muted); border-color: var(--line); }
  .actions button:hover { filter: brightness(0.95); }
</style>
