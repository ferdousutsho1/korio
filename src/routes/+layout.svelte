<script lang="ts">
  import "$lib/tokens.css";
  import { onMount } from "svelte";
  import { initTheme } from "$lib/theme";
  import { initSidebar } from "$lib/sidebar";
  import { initSync } from "$lib/sync";
  import { initDigest } from "$lib/digest";
  let { children } = $props();
  onMount(() => { initTheme(); initSidebar(); initSync(); initDigest(); });
</script>

{@render children()}
