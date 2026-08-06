<script lang="ts">
  import "$lib/tokens.css";
  import { onMount } from "svelte";
  import { initTheme } from "$lib/theme";
  import { initSidebar } from "$lib/sidebar";
  import { initSync } from "$lib/sync";
  import { initDigest } from "$lib/digest";
  import { initReminders } from "$lib/reminders";
  let { children } = $props();
  onMount(() => { initTheme(); initSidebar(); initSync(); initDigest(); initReminders(); });
</script>

{@render children()}
