<script lang="ts">
  import { onMount } from "svelte";
  import { goalsProgress, addGoal, updateGoal, deleteGoal, listApps, type GoalProgress, type App } from "$lib/api";
  import { goalLabel, formatTarget } from "$lib/goals";

  let rows = $state<GoalProgress[]>([]);
  let apps = $state<App[]>([]);

  // new-goal form
  let scope = $state<"kind" | "app" | "total">("kind");
  let kindRef = $state("productive");
  let appRef = $state<string>("");
  let comparator = $state<"gte" | "lte">("gte");
  let targetMin = $state(120);

  const appName = (ref: string | null) =>
    apps.find((a) => String(a.id) === ref)?.display_name ?? "app";

  async function refresh() { rows = await goalsProgress(); }
  onMount(async () => { apps = await listApps(); await refresh(); });

  async function add() {
    const secs = Math.max(1, Math.round(targetMin)) * 60;
    const ref = scope === "kind" ? kindRef : scope === "app" ? appRef : null;
    if (scope === "app" && !ref) return;
    await addGoal({ scope, scopeRef: ref, comparator, targetSeconds: secs });
    await refresh();
  }
  async function bumpTarget(r: GoalProgress, deltaMin: number) {
    const secs = Math.max(60, r.goal.target_seconds + deltaMin * 60);
    await updateGoal(r.goal.id, r.goal.comparator, secs);
    await refresh();
  }
  async function flipComparator(r: GoalProgress) {
    await updateGoal(r.goal.id, r.goal.comparator === "gte" ? "lte" : "gte", r.goal.target_seconds);
    await refresh();
  }
  async function remove(r: GoalProgress) { await deleteGoal(r.goal.id); await refresh(); }

  const pct = (r: GoalProgress) =>
    Math.min(100, Math.round((r.today_seconds / Math.max(1, r.goal.target_seconds)) * 100));
</script>

<div class="goals">
  <form class="add" onsubmit={(e) => { e.preventDefault(); add(); }}>
    <select bind:value={scope} aria-label="Goal scope">
      <option value="kind">Kind</option>
      <option value="app">App</option>
      <option value="total">Total</option>
    </select>
    {#if scope === "kind"}
      <select bind:value={kindRef} aria-label="Kind">
        <option value="productive">Productive</option>
        <option value="neutral">Neutral</option>
        <option value="distracting">Distracting</option>
      </select>
    {:else if scope === "app"}
      <select bind:value={appRef} aria-label="App">
        <option value="" disabled>Choose app…</option>
        {#each apps as a}<option value={String(a.id)}>{a.display_name}</option>{/each}
      </select>
    {/if}
    <select bind:value={comparator} aria-label="Direction">
      <option value="gte">at least ≥</option>
      <option value="lte">at most ≤</option>
    </select>
    <span class="num"><input type="number" min="1" bind:value={targetMin} aria-label="Target minutes" /><span>min</span></span>
    <button type="submit" class="addbtn">Add goal</button>
  </form>

  {#if rows.length === 0}
    <div class="empty">No goals yet. Set a daily target to start a streak.</div>
  {:else}
    <ul class="list">
      {#each rows as r (r.goal.id)}
        <li class:met={r.met_today}>
          <div class="head">
            <span class="label">{goalLabel(r.goal, appName(r.goal.scope_ref))}</span>
            <span class="streak" title="Current / best streak">🔥 {r.current_streak}<span class="best"> · best {r.best_streak}</span></span>
            <button class="x" onclick={() => remove(r)} aria-label="Delete goal" title="Delete">×</button>
          </div>
          <div class="bar"><div class="fill" style="width:{pct(r)}%"></div></div>
          <div class="foot">
            <span class="today">{formatTarget(r.today_seconds)} today {r.met_today ? "· met ✓" : ""}</span>
            <span class="edit">
              <button onclick={() => bumpTarget(r, -15)} aria-label="Decrease target">−15m</button>
              <button onclick={() => bumpTarget(r, 15)} aria-label="Increase target">+15m</button>
              <button onclick={() => flipComparator(r)} title="Toggle ≥/≤">{r.goal.comparator === "gte" ? "≥" : "≤"}</button>
            </span>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .goals { max-width: 640px; display: flex; flex-direction: column; gap: 16px; }
  .add { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .add select, .num input { font: inherit; font-size: 13px; padding: 8px 10px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--surface); color: var(--text); }
  .num { display: flex; align-items: center; gap: 6px; color: var(--muted); font-size: 12px; }
  .num input { width: 72px; }
  .addbtn { font: inherit; padding: 9px 16px; border: none; border-radius: var(--radius-sm);
    background: var(--accent); color: var(--accent-contrast); cursor: pointer; }
  .empty { color: var(--muted); border: 1px dashed var(--line); border-radius: var(--radius); padding: 40px; text-align: center; }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
  .list li { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 12px 14px;
    display: flex; flex-direction: column; gap: 8px; }
  .list li.met { border-color: color-mix(in srgb, var(--accent) 55%, var(--line)); }
  .head { display: flex; align-items: center; gap: 10px; }
  .label { font-weight: 600; flex: 1; }
  .streak { font-size: 13px; color: var(--muted); }
  .best { color: var(--muted); }
  .x { border: none; background: none; color: var(--muted); font-size: 18px; cursor: pointer; line-height: 1; }
  .bar { height: 8px; background: var(--chip); border-radius: 999px; overflow: hidden; }
  .fill { height: 100%; background: var(--accent); border-radius: 999px; }
  .foot { display: flex; justify-content: space-between; align-items: center; color: var(--muted); font-size: 12px; }
  .edit { display: flex; gap: 6px; }
  .edit button { font: inherit; font-size: 12px; padding: 4px 8px; border: 1px solid var(--line);
    border-radius: var(--radius-sm); background: var(--bg); color: var(--text); cursor: pointer; }
</style>
