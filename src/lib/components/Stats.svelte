<script lang="ts">
  import RangePicker from "$lib/components/RangePicker.svelte";
  import BarBreakdown from "$lib/components/BarBreakdown.svelte";
  import Heatmap from "$lib/components/Heatmap.svelte";
  import TimelineRibbon from "$lib/components/TimelineRibbon.svelte";
  import { usageRange, dailyTotals, daySessions, type UsageSlice, type DayTotal, type SessionRow } from "$lib/api";
  import { presetRange, customRange, dayStartLocal, isoDate, type Range, type Preset } from "$lib/ranges";
  import { readJSON, writeJSON } from "$lib/prefs";

  type SavedRange =
    | { kind: "preset"; preset: Preset; label: string }
    | { kind: "custom"; from: string; to: string; label: string };
  const KEY = "korio.stats.range";
  const saved = readJSON<SavedRange>(KEY, { kind: "preset", preset: "today", label: "Today" });
  const initialRange: Range = saved.kind === "preset" ? presetRange(saved.preset) : customRange(saved.from, saved.to);
  const initialPicker = saved.kind === "preset" ? saved.preset : { from: saved.from, to: saved.to };
  let range = $state<Range>(initialRange);
  let rangeLabel = $state(saved.label);
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

  function onRange(r: Range, label: string) {
    range = r;
    rangeLabel = label;
    // Recover the preset from the label set by RangePicker; custom shows "a → b".
    const preset = (["Today", "Last 7 days", "Last 30 days", "This month"] as const)
      .indexOf(label as any) >= 0
      ? ({ Today: "today", "Last 7 days": "7d", "Last 30 days": "30d", "This month": "month" } as const)[label as "Today"]
      : null;
    if (preset) writeJSON(KEY, { kind: "preset", preset, label });
    else writeJSON(KEY, { kind: "custom", from: isoDate(r.from), to: isoDate(r.to - 86400), label });
  }
  function onPickDay(day: number) { selectedDay = day; }
</script>

<div class="stats">
  <RangePicker onChange={onRange} initial={initialPicker} />
  <div class="sub">{rangeLabel}</div>
  <BarBreakdown {slices} />
  <Heatmap from={range.from} to={range.to} {totals} {onPickDay} />
  <TimelineRibbon dayStart={selectedDay} sessions={daySess} />
</div>

<style>
  .stats { display: flex; flex-direction: column; gap: 16px; }
  .sub { font-size: 12px; color: var(--muted); margin: -6px 0 0; }
</style>
