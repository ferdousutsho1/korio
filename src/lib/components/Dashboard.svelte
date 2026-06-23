<script lang="ts">
  import { loadLayout, saveLayout, type CardState, type CardId } from "$lib/dashboard";
  import DashboardCustomize from "$lib/components/DashboardCustomize.svelte";
  import FocusBreakdownCard from "$lib/components/cards/FocusBreakdownCard.svelte";
  import FocusScoreCard from "$lib/components/cards/FocusScoreCard.svelte";
  import TrackingNowCard from "$lib/components/cards/TrackingNowCard.svelte";
  import TodayStatsCard from "$lib/components/cards/TodayStatsCard.svelte";
  import TasksCard from "$lib/components/cards/TasksCard.svelte";
  import NotesCard from "$lib/components/cards/NotesCard.svelte";
  import GoalsCard from "$lib/components/cards/GoalsCard.svelte";
  import CategoryBreakdownCard from "$lib/components/cards/CategoryBreakdownCard.svelte";

  const COMPONENTS: Record<CardId, any> = {
    focus_breakdown: FocusBreakdownCard, category_breakdown: CategoryBreakdownCard,
    focus_score: FocusScoreCard, tracking_now: TrackingNowCard,
    today_stats: TodayStatsCard, tasks: TasksCard, notes: NotesCard, goals: GoalsCard,
  };

  let layout = $state<CardState[]>(loadLayout());
  let editing = $state(false);
  function apply(next: CardState[]) { layout = next; saveLayout(next); }
  let visible = $derived(layout.filter((c) => c.enabled));
</script>

<div class="bar">
  <button class="customize" onclick={() => (editing = true)}>⚙ Customize</button>
</div>

<div class="grid">
  {#each visible as c (c.id)}
    {@const Card = COMPONENTS[c.id]}
    {#if Card}<div class="cell" class:wide={c.id === "focus_breakdown" || c.id === "category_breakdown"}><Card /></div>{/if}
  {/each}
</div>

{#if editing}
  <DashboardCustomize {layout} onChange={apply} onClose={() => (editing = false)} />
{/if}

<style>
  .bar { display: flex; justify-content: flex-end; margin-bottom: 14px; }
  .customize { font: inherit; font-size: 13px; padding: 7px 14px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--muted); cursor: pointer; }
  .customize:hover { color: var(--text); border-color: var(--accent); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px; align-items: stretch; }
  .cell { display: flex; }
  .cell > :global(*) { width: 100%; }
  .cell.wide { grid-column: span 2; }
  @media (max-width: 720px) { .cell.wide { grid-column: span 1; } }
  .cell > :global(.card) { height: 100%; box-sizing: border-box; }
</style>
