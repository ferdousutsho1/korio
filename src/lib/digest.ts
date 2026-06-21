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

// ---- Scheduler (main window only; no-ops without Tauri) ----
import { browser } from "$app/environment";
import { isMainWindow } from "$lib/sync";
import { getSettings, setSetting, usageToday, scoreToday, goalsProgress } from "$lib/api";
import { navIntent } from "$lib/nav";

let started = false;

/** Start the once-a-day digest scheduler (main window only; safe to call without Tauri). */
export function initDigest() {
  if (!browser || started) return;
  started = true;
  setTimeout(check, 3_000);          // shortly after launch
  setInterval(check, 30_000);        // then every 30s
  registerClickHandler();
}

async function check() {
  if (!isMainWindow()) return;
  let s: Record<string, string>;
  try { s = await getSettings(); } catch { return; } // not in Tauri
  if (s.digest_enabled !== "true") return;
  const target = parseHm(s.digest_time || "18:00");
  if (target === null) return;
  const now = new Date();
  const today = ymd(now);
  if (!shouldSend(true, minutesOfDay(now), target, s.digest_last_sent || "", today)) return;
  await sendDigest();
  await setSetting("digest_last_sent", today);
}

async function sendDigest() {
  try {
    const [usage, score, goals] = await Promise.all([usageToday(), scoreToday(), goalsProgress()]);
    const total = usage.reduce((a, u) => a + u.seconds, 0);
    const top = usage[0] ?? null; // usageToday is sorted DESC
    const goalsMet = goals.filter((g) => g.met_today).length;
    const { title, body } = composeDigest({
      totalSeconds: total, score,
      topName: top?.display_name ?? null, topSeconds: top?.seconds ?? 0,
      goalsMet, goalsTotal: goals.length,
    });
    const notif = await import("@tauri-apps/plugin-notification");
    let granted = await notif.isPermissionGranted();
    if (!granted) granted = (await notif.requestPermission()) === "granted";
    if (granted) notif.sendNotification({ title, body });
  } catch { /* notification unavailable */ }
}

/** Best-effort: when the toast is activated, focus the window and route to Dashboard. */
async function registerClickHandler() {
  try {
    const notif = await import("@tauri-apps/plugin-notification");
    if (typeof notif.onAction === "function") {
      await notif.onAction(async () => {
        navIntent.set("dashboard");
        try {
          const { getCurrentWindow } = await import("@tauri-apps/api/window");
          const w = getCurrentWindow();
          await w.show(); await w.setFocus();
        } catch { /* ignore */ }
      });
    }
  } catch { /* onAction unsupported — toast still shows */ }
}

/** Ask the OS for notification permission; returns true if granted. */
export async function ensureNotificationPermission(): Promise<boolean> {
  try {
    const notif = await import("@tauri-apps/plugin-notification");
    if (await notif.isPermissionGranted()) return true;
    return (await notif.requestPermission()) === "granted";
  } catch { return false; }
}
