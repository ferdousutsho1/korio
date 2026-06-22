<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Watchlist from "$lib/components/Watchlist.svelte";
  import Dashboard from "$lib/components/Dashboard.svelte";
  import Stats from "$lib/components/Stats.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import Tools from "$lib/components/Tools.svelte";
  import Tasks from "$lib/components/Tasks.svelte";
  import Notes from "$lib/components/Notes.svelte";
  import Goals from "$lib/components/Goals.svelte";
  import Sites from "$lib/components/Sites.svelte";
  import LimitWarning from "$lib/components/LimitWarning.svelte";
  import LockScreen from "$lib/components/LockScreen.svelte";
  import { theme, toggleTheme } from "$lib/theme";
  import { browser } from "$app/environment";
  import { onMount } from "svelte";
  import { navIntent } from "$lib/nav";
  import CaptureWindow from "$lib/components/CaptureWindow.svelte";
  const captureMode = browser && new URLSearchParams(location.search).has("capture");
  let active = $state("dashboard");
  onMount(() => navIntent.subscribe((v) => { if (v) { active = v; navIntent.set(null); } }));
</script>

{#if captureMode}
<CaptureWindow />
{:else}
<main class="shell">
  <Sidebar {active} onNavigate={(id) => (active = id)} />
  <section class="content">
    <header class="topbar">
      <h1>{active === "dashboard" ? "Dashboard" : active === "stats" ? "Stats" : active === "tools" ? "Tools" : active === "tasks" ? "Tasks" : active === "notes" ? "Notes" : active === "goals" ? "Goals" : active === "sites" ? "Sites" : active === "settings" ? "Settings" : "Watchlist"}</h1>
      <button class="theme" aria-label={$theme === "light" ? "Switch to dark theme" : "Switch to light theme"} onclick={toggleTheme}>{$theme === "light" ? "☾" : "☀"}</button>
    </header>
    <div class="view">
      {#if active === "dashboard"}
        <Dashboard />
      {:else if active === "stats"}
        <Stats />
      {:else if active === "tools"}
        <Tools />
      {:else if active === "tasks"}
        <Tasks />
      {:else if active === "notes"}
        <Notes />
      {:else if active === "goals"}
        <Goals />
      {:else if active === "sites"}
        <Sites />
      {:else if active === "settings"}
        <Settings />
      {:else}
        <Watchlist />
      {/if}
    </div>
  </section>
  <LimitWarning />
  <LockScreen />
</main>
{/if}

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
</style>
