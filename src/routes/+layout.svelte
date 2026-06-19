<script lang="ts">
  import "$lib/tokens.css";
  import { onMount } from "svelte";
  import { initTheme } from "$lib/theme";
  let { children } = $props();
  onMount(initTheme);
</script>

{@render children()}
