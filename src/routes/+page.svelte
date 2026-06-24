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
  import { browser } from "$app/environment";
  import { onMount } from "svelte";
  import { navIntent } from "$lib/nav";
  import { hiddenSections } from "$lib/sidebar";
  import CaptureWindow from "$lib/components/CaptureWindow.svelte";
  const captureMode = browser && new URLSearchParams(location.search).has("capture");
  let active = $state("dashboard");
  onMount(() => navIntent.subscribe((v) => { if (v) { active = v; navIntent.set(null); } }));
  $effect(() => {
    if ($hiddenSections.includes(active)) {
      const order = ["dashboard","stats","watchlist","sites","tools","tasks","notes","goals","settings"];
      active = order.find((id) => !$hiddenSections.includes(id)) ?? "settings";
    }
  });
</script>

{#if captureMode}
<CaptureWindow />
{:else}
<main class="shell">
  <Sidebar {active} onNavigate={(id) => (active = id)} />
  <section class="content">
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
  .view { padding: 28px; overflow: auto; }
</style>
