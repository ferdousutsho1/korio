<script lang="ts">
  import { onMount } from "svelte";
  import { usageRange, scoreRange, siteSlices, listApps, listSiteCaps, listTasks,
    usageByCategory, type UsageSlice, type SiteSlice, type CategoryUsage } from "$lib/api";
  import { formatDuration } from "$lib/format";
  import { digestQuips, markDigestViewed, subjectDayBounds, subjectDayLabel, previousDay,
    type Highlights, type Overage } from "$lib/digest";
  import DonutChart from "$lib/components/DonutChart.svelte";
  import { readJSON } from "$lib/prefs";

  let loading = $state(true);
  let apps = $state<UsageSlice[]>([]);
  let sites = $state<SiteSlice[]>([]);
  let cats = $state<CategoryUsage[]>([]);
  let score = $state(0);
  let tasksDone = $state(0);
  let tasksTotal = $state(0);
  let overages = $state<Overage[]>([]);
  let pomodoros = $state(0);

  // The digest always covers the day that has finished, so it's never built from
  // a day still in progress (at 00:05 that would be an empty five minutes).
  const now = new Date();

  async function load() {
    const [from, to] = subjectDayBounds(now);
    const [u, s, sl, cu, appRows, caps, tasks] = await Promise.all([
      usageRange(from, to), scoreRange(from, to), siteSlices(from, to), usageByCategory(from, to),
      listApps(), listSiteCaps(), listTasks(from, to),
    ]);
    apps = u; score = s; sites = sl; cats = cu;
    tasksTotal = tasks.length;
    tasksDone = tasks.filter((t) => t.done).length;

    const usedByApp = new Map(u.map((x) => [x.app_id, x.seconds]));
    const usedBySite = new Map(sl.map((x) => [x.domain, x.seconds]));
    const over: Overage[] = [];
    for (const a of appRows) {
      const used = usedByApp.get(a.id) ?? 0;
      if (a.daily_cap_seconds > 0 && used > a.daily_cap_seconds) {
        over.push({ name: a.display_name, kind: "app", capSeconds: a.daily_cap_seconds, usedSeconds: used });
      }
    }
    for (const c of caps) {
      const used = usedBySite.get(c.domain) ?? 0;
      if (c.daily_cap_seconds > 0 && used > c.daily_cap_seconds) {
        over.push({ name: c.display_name || c.domain, kind: "site", capSeconds: c.daily_cap_seconds, usedSeconds: used });
      }
    }
    overages = over;

    // Pomodoro keeps its own per-day counter in localStorage ({date, count}).
    // It isn't cleared at midnight, only overwritten on the next completed session,
    // so the finished day's count is still readable for most of the following day.
    const p = readJSON<{ date?: string; count?: number }>("korio.pomodoro.count", {});
    const subjectStr = previousDay(now).toDateString();  // must match pomodoro.ts's key format
    pomodoros = p?.date === subjectStr && typeof p.count === "number" ? p.count : 0;

    loading = false;
  }

  onMount(async () => {
    await load();
    await markDigestViewed();   // opening the tab clears the glow for this subject day
  });

  let appTotal = $derived(apps.reduce((a, x) => a + x.seconds, 0));
  let siteTotal = $derived(sites.reduce((a, x) => a + x.seconds, 0));
  let appMax = $derived(apps.reduce((m, x) => Math.max(m, x.seconds), 0));
  let siteMax = $derived(sites.reduce((m, x) => Math.max(m, x.seconds), 0));
  let topApps = $derived(apps.slice(0, 5));
  let topSites = $derived(sites.slice(0, 5));

  let highlights = $derived<Highlights>({
    totalSeconds: appTotal, score,
    topName: apps[0]?.display_name ?? null, topSeconds: apps[0]?.seconds ?? 0,
    topSiteName: sites[0] ? (sites[0].display_name || sites[0].domain) : null,
    topSiteSeconds: sites[0]?.seconds ?? 0,
    tasksDone, tasksTotal, pomodoros, overages,
  });
  let quips = $derived(digestQuips(highlights));

  let dayLabel = subjectDayLabel(now);
  let nothingTracked = $derived(!loading && appTotal === 0 && siteTotal === 0 && tasksTotal === 0);
</script>

<div class="digest">
  <header class="hero">
    <div class="eyebrow">End of day · {dayLabel}</div>
    <h2>{formatDuration(appTotal)} focused</h2>
    <div class="stats">
      <span><strong>{score}</strong> focus score</span>
      <span><strong>{formatDuration(siteTotal)}</strong> on sites</span>
      <span><strong>{tasksDone}/{tasksTotal}</strong> to-dos done</span>
      {#if pomodoros > 0}<span><strong>{pomodoros}</strong> pomodoros</span>{/if}
    </div>
  </header>

  {#if loading}
    <p class="empty">Building the digest…</p>
  {:else if nothingTracked}
    <div class="none">
      <p>Korio didn't track anything on {dayLabel}.</p>
      <p class="sub">The digest covers the day that just ended, so today's activity shows up
        in tomorrow's — and the tab glows when it's ready.</p>
    </div>
  {:else}
    {#if quips.length}
      <ul class="quips">
        {#each quips as q}<li>{q}</li>{/each}
      </ul>
    {/if}

    <div class="grid">
      <section class="card">
        <div class="label">Top apps</div>
        {#if topApps.length === 0}
          <p class="empty">No app time tracked today.</p>
        {:else}
          <ul class="bars">
            {#each topApps as a (a.app_id)}
              <li>
                <span class="n" title={a.display_name}>{a.display_name}</span>
                <span class="track"><span class="fill"
                  style={`width:${appMax ? (a.seconds / appMax) * 100 : 0}%;background:${a.color}`}></span></span>
                <span class="t">{formatDuration(a.seconds)}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="card">
        <div class="label">Top sites</div>
        {#if topSites.length === 0}
          <p class="empty">No site time tracked today.</p>
        {:else}
          <ul class="bars">
            {#each topSites as s (s.domain)}
              <li>
                <span class="n" title={s.domain}>{s.display_name || s.domain}</span>
                <span class="track"><span class="fill"
                  style={`width:${siteMax ? (s.seconds / siteMax) * 100 : 0}%;background:${s.color}`}></span></span>
                <span class="t">{formatDuration(s.seconds)}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="card wide">
        <div class="label">Where the day went</div>
        {#if cats.length === 0}
          <p class="empty">Nothing categorised yet — assign categories in Watchlist or Sites.</p>
        {:else}
          <div class="chartrow">
            <DonutChart slices={cats.map((c) => ({ display_name: c.name, color: c.color, seconds: c.seconds }))}
              inner={0.6} />
            <ul class="legend">
              {#each cats as c (c.category_id ?? -1)}
                <li><span class="sw" style={`background:${c.color}`}></span>
                  <span class="n">{c.name}</span>
                  <span class="t">{formatDuration(c.seconds)}</span></li>
              {/each}
            </ul>
          </div>
        {/if}
      </section>

      {#if overages.length}
        <section class="card wide over">
          <div class="label">Limits blown through</div>
          <ul class="overs">
            {#each overages as o (o.kind + o.name)}
              <li>
                <span class="n">{o.name}</span>
                <span class="tag">{o.kind}</span>
                <span class="t">{formatDuration(o.usedSeconds - o.capSeconds)} over {formatDuration(o.capSeconds)}</span>
              </li>
            {/each}
          </ul>
        </section>
      {/if}
    </div>
  {/if}
</div>

<style>
  .digest { max-width: 860px; display: flex; flex-direction: column; gap: 18px; }
  .hero { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 24px; }
  .eyebrow { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--accent); }
  .hero h2 { font-family: var(--font-display); font-size: 34px; margin: 8px 0 12px; letter-spacing: -.5px; }
  .stats { display: flex; gap: 22px; flex-wrap: wrap; font-size: 13px; color: var(--muted); }
  .stats strong { color: var(--text); font-variant-numeric: tabular-nums; }

  .quips { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px; }
  .quips li { background: color-mix(in srgb, var(--accent) 8%, var(--surface));
    border-left: 3px solid var(--accent); border-radius: var(--radius-sm);
    padding: 11px 14px; font-size: 13px; color: var(--text); }

  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 16px; align-items: stretch; }
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 18px; }
  .card.wide { grid-column: 1 / -1; }
  .card.over { border-color: var(--warn-close); }
  .label { font-size: 10px; letter-spacing: 2px; text-transform: uppercase; color: var(--muted); margin-bottom: 14px; }
  .empty { color: var(--muted); font-size: 13px; margin: 0; }
  .none { border: 1px dashed var(--line); border-radius: var(--radius); padding: 36px; text-align: center; }
  .none p { margin: 0; font-size: 14px; }
  .none .sub { color: var(--muted); font-size: 12.5px; margin-top: 10px; max-width: 460px;
    margin-left: auto; margin-right: auto; line-height: 1.6; }

  .bars { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
  .bars li { display: grid; grid-template-columns: 110px 1fr auto; align-items: center; gap: 10px; font-size: 12px; }
  .n { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .track { height: 10px; background: color-mix(in srgb, var(--text) 6%, transparent);
    border-radius: 5px; overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 5px; min-width: 2px; }
  .t { color: var(--muted); font-variant-numeric: tabular-nums; }

  .chartrow { display: flex; gap: 24px; align-items: center; flex-wrap: wrap; }
  .legend { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 9px; flex: 1; min-width: 180px; }
  .legend li { display: flex; align-items: center; gap: 10px; font-size: 13px; }
  .sw { width: 10px; height: 10px; border-radius: 3px; flex-shrink: 0; }
  .legend .t { margin-left: auto; }

  .overs { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px; }
  .overs li { display: flex; align-items: center; gap: 10px; font-size: 13px; }
  .overs .t { margin-left: auto; color: var(--warn-close); }
  .tag { font-size: 10px; letter-spacing: 1px; text-transform: uppercase; color: var(--muted);
    border: 1px solid var(--line); border-radius: 999px; padding: 1px 7px; }
</style>
