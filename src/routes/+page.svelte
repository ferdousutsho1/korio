<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { theme, toggleTheme } from "$lib/theme";
  let active = $state("dashboard");
</script>

<main class="shell">
  <Sidebar {active} onNavigate={(id) => (active = id)} />
  <section class="content">
    <header class="topbar">
      <h1>{active === "dashboard" ? "Dashboard" : "Watchlist"}</h1>
      <button class="theme" onclick={toggleTheme}>{$theme === "light" ? "☾" : "☀"}</button>
    </header>
    <div class="view">
      {#if active === "dashboard"}
        <p class="muted">Dashboard coming in Task 10.</p>
      {:else}
        <p class="muted">Watchlist coming in Task 8.</p>
      {/if}
    </div>
  </section>
</main>

<style>
  :global(html), :global(body) { margin: 0; height: 100%; }
  :global(body) { background: var(--bg); color: var(--text); font-family: var(--font-body); }
  .shell { display: flex; height: 100vh; }
  .content { flex: 1; display: flex; flex-direction: column; min-width: 0; }
  .topbar { display: flex; align-items: center; justify-content: space-between;
    padding: 20px 28px; border-bottom: 1px solid var(--line); }
  h1 { font-family: var(--font-display); font-weight: 600; font-size: 26px; margin: 0;
    letter-spacing: -.4px; }
  .theme { width: 38px; height: 38px; border-radius: var(--radius-sm); border: 1px solid var(--line);
    background: var(--surface); color: var(--text); cursor: pointer; font-size: 16px; }
  .view { padding: 28px; overflow: auto; }
  .muted { color: var(--muted); }
</style>
