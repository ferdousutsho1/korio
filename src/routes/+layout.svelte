<script lang="ts">
  import "$lib/tokens.css";
  import { onMount } from "svelte";
  import { initTheme } from "$lib/theme";
  import { initSidebar } from "$lib/sidebar";
  let { children } = $props();
  onMount(() => { initTheme(); initSidebar(); });
</script>

{@render children()}
