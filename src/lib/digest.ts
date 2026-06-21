import { formatDuration } from "$lib/format";

/** Minutes since local midnight. */
export function minutesOfDay(d: Date): number {
  return d.getHours() * 60 + d.getMinutes();
}

/** "HH:MM" → minutes since midnight, or null if malformed/out of range. */
export function parseHm(hm: string): number | null {
  const m = /^(\d{2}):(\d{2})$/.exec(hm);
  if (!m) return null;
  const h = Number(m[1]);
  const min = Number(m[2]);
  if (h > 23 || min > 59) return null;
  return h * 60 + min;
}

/** Local YYYY-MM-DD. */
export function ymd(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/** Fire when enabled, the local time has reached the target, and we haven't sent today. */
export function shouldSend(enabled: boolean, nowMinutes: number, targetMinutes: number, lastSent: string, today: string): boolean {
  return enabled && nowMinutes >= targetMinutes && lastSent !== today;
}

export interface DigestStats {
  totalSeconds: number;
  score: number;
  topName: string | null;
  topSeconds: number;
  goalsMet: number;
  goalsTotal: number;
}

/** Build the notification title + body from the day's stats. */
export function composeDigest(s: DigestStats): { title: string; body: string } {
  const lines = [`${formatDuration(s.totalSeconds)} focused · score ${s.score}`];
  if (s.topName) lines.push(`Top: ${s.topName} ${formatDuration(s.topSeconds)}`);
  if (s.goalsTotal > 0) lines.push(`${s.goalsMet}/${s.goalsTotal} goals met`);
  return { title: "Korio — today's recap", body: lines.join("\n") };
}
