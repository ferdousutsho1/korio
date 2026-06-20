<script lang="ts">
  import RangePicker from "$lib/components/RangePicker.svelte";
  import BarBreakdown from "$lib/components/BarBreakdown.svelte";
  import Heatmap from "$lib/components/Heatmap.svelte";
  import TimelineRibbon from "$lib/components/TimelineRibbon.svelte";
  import { usageRange, dailyTotals, daySessions, type UsageSlice, type DayTotal, type SessionRow } from "$lib/api";
  import { presetRange, dayStartLocal, type Range } from "$lib/ranges";

  let range = $state<Range>(presetRange("7d"));
  let rangeLabel = $state("Last 7 days");
  let slices = $state<UsageSlice[]>([]);
  let totals = $state<DayTotal[]>([]);
  let selectedDay = $state<number>(dayStartLocal(new Date()));
  let daySess = $state<SessionRow[]>([]);

  async function loadRange() {
    [slices, totals] = await Promise.all([usageRange(range.from, range.to), dailyTotals(range.from, range.to)]);
  }
  async function loadDay() {
    daySess = await daySessions(selectedDay, selectedDay + 86400);
  }
  $effect(() => { range; loadRange(); });
  $effect(() => { selectedDay; loadDay(); });

  function onRange(r: Range, label: string) { range = r; rangeLabel = label; }
  function onPickDay(day: number) { selectedDay = day; }
</script>

<div class="stats">
  <RangePicker onChange={onRange} />
  <div class="sub">{rangeLabel}</div>
  <BarBreakdown {slices} />
  <Heatmap from={range.from} to={range.to} {totals} {onPickDay} />
  <TimelineRibbon dayStart={selectedDay} sessions={daySess} />
</div>

<style>
  .stats { display: flex; flex-direction: column; gap: 16px; }
  .sub { font-size: 12px; color: var(--muted); margin: -6px 0 0; }
</style>
