export type Preset = "today" | "7d" | "30d" | "month";
export interface Range { from: number; to: number; }

const DAY = 86400;
const toUnix = (d: Date) => Math.floor(d.getTime() / 1000);

/** Local midnight (unix seconds) for the day containing `d`. */
export function dayStartLocal(d: Date): number {
  return toUnix(new Date(d.getFullYear(), d.getMonth(), d.getDate(), 0, 0, 0));
}

/** A preset range as [from, to) unix seconds, relative to `now`. */
export function presetRange(preset: Preset, now: Date = new Date()): Range {
  const todayStart = dayStartLocal(now);
  const tomorrow = todayStart + DAY;
  switch (preset) {
    case "today": return { from: todayStart, to: tomorrow };
    case "7d": return { from: tomorrow - 7 * DAY, to: tomorrow };
    case "30d": return { from: tomorrow - 30 * DAY, to: tomorrow };
    case "month": {
      const first = toUnix(new Date(now.getFullYear(), now.getMonth(), 1, 0, 0, 0));
      return { from: first, to: tomorrow };
    }
  }
}

/** Build [from,to) from two yyyy-mm-dd strings (inclusive of the end day). */
export function customRange(fromStr: string, toStr: string): Range {
  const [fy, fm, fd] = fromStr.split("-").map(Number);
  const [ty, tm, td] = toStr.split("-").map(Number);
  const from = toUnix(new Date(fy, fm - 1, fd, 0, 0, 0));
  const to = toUnix(new Date(ty, tm - 1, td, 0, 0, 0)) + DAY;
  return { from, to };
}

/** 0 (none) then 1..4 by proportion of `max`. */
export function heatLevel(seconds: number, max: number): number {
  if (seconds <= 0 || max <= 0) return 0;
  const r = seconds / max;
  if (r > 0.75) return 4;
  if (r > 0.5) return 3;
  if (r > 0.25) return 2;
  return 1;
}

/** Inclusive list of LOCAL day-starts from `from` to `< to`, incremented by
 *  calendar day so keys match the backend's local-midnight buckets in all zones. */
export function eachDay(from: number, to: number): number[] {
  const out: number[] = [];
  let cur = new Date(from * 1000);
  let t = dayStartLocal(cur);
  while (t < to) {
    out.push(t);
    cur = new Date(cur.getFullYear(), cur.getMonth(), cur.getDate() + 1, 0, 0, 0);
    t = toUnix(cur);
  }
  return out;
}

/** yyyy-mm-dd for a unix day-start (for <input type=date> values). */
export function isoDate(unix: number): string {
  const d = new Date(unix * 1000);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}
