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
}

/** Build the notification title + body from the day's stats. */
export function composeDigest(s: DigestStats): { title: string; body: string } {
  const lines = [`${formatDuration(s.totalSeconds)} focused · score ${s.score}`];
  if (s.topName) lines.push(`Top: ${s.topName} ${formatDuration(s.topSeconds)}`);
  return { title: "Korio — today's recap", body: lines.join("\n") };
}

// ---- End-of-day digest highlights ----

export interface Overage {
  name: string;
  kind: "app" | "site";
  capSeconds: number;
  usedSeconds: number;
}

export interface Highlights {
  totalSeconds: number;
  score: number;
  topName: string | null;
  topSeconds: number;
  topSiteName: string | null;
  topSiteSeconds: number;
  tasksDone: number;
  tasksTotal: number;
  pomodoros: number;
  overages: Overage[];
}

/**
 * Short, slightly cheeky commentary on the day — at most three lines, most
 * pointed first. Pure and deterministic so it can be snapshot-tested.
 */
export function digestQuips(h: Highlights): string[] {
  const out: string[] = [];
  const worst = [...h.overages].sort(
    (a, b) => b.usedSeconds - b.capSeconds - (a.usedSeconds - a.capSeconds),
  )[0];

  if (worst) {
    const over = formatDuration(worst.usedSeconds - worst.capSeconds);
    const ratio = worst.capSeconds > 0 ? worst.usedSeconds / worst.capSeconds : 1;
    if (ratio >= 3) {
      out.push(`${worst.name} ran ${over} past its limit. At this point the limit is just a suggestion you've heard of.`);
    } else if (ratio >= 2) {
      out.push(`You doubled ${worst.name}'s limit — ${over} over. Bold.`);
    } else {
      out.push(`${worst.name} went ${over} over its limit. Close, though.`);
    }
    if (h.overages.length > 1) {
      out.push(`${h.overages.length} limits ignored today. Impressive commitment to the bit.`);
    }
  }

  if (h.totalSeconds === 0) {
    out.push("Nothing tracked today. Either a proper rest day, or Korio took one.");
  } else if (h.score >= 85 && !worst) {
    out.push(`Focus score ${h.score}. Suspiciously productive — who are you and what did you do with yourself?`);
  } else if (h.score <= 35) {
    out.push(`Focus score ${h.score}. Tomorrow's a fresh scoreboard.`);
  }

  if (h.tasksTotal > 0 && h.tasksDone === h.tasksTotal) {
    out.push("Every to-do checked off. Show-off.");
  } else if (h.tasksTotal > 0 && h.tasksDone === 0) {
    out.push(`${h.tasksTotal} to-do${h.tasksTotal === 1 ? "" : "s"} still waiting patiently.`);
  }

  if (out.length === 0 && h.topName) {
    out.push(`${h.topName} took the crown at ${formatDuration(h.topSeconds)}.`);
  }
  return out.slice(0, 3);
}

/**
 * Whether Snooze/Ignore on a limit alert must be unlocked with the PIN.
 * Requires BOTH the setting and an actual PIN — otherwise the gate would be
 * armed but unopenable (and `verify_pin` fails open when no PIN is stored).
 */
export function limitPinRequired(settings: Record<string, string>, hasPin: boolean): boolean {
  return settings.limit_pin_enabled === "true" && hasPin;
}

/** The day's digest is ready once the local clock passes the configured time. */
export function isDigestReady(nowMinutes: number, targetMinutes: number): boolean {
  return nowMinutes >= targetMinutes;
}

/** Unread = a digest is ready for today and today's hasn't been opened yet. */
export function isDigestUnread(ready: boolean, lastViewed: string, today: string): boolean {
  return ready && lastViewed !== today;
}

// ---- Scheduler (main window only; no-ops without Tauri) ----
import { browser } from "$app/environment";
import { writable } from "svelte/store";
import { isMainWindow } from "$lib/sync";
import { getSettings, setSetting, usageToday, scoreToday } from "$lib/api";
import { navIntent } from "$lib/nav";

let started = false;

/** True while today's digest is ready but hasn't been opened — drives the sidebar glow. */
export const digestUnread = writable(false);

/** Re-read the digest schedule and recompute the unread flag. Safe without Tauri. */
export async function refreshDigestUnread() {
  try {
    const s = await getSettings();
    const target = parseHm(s.digest_time || "18:00") ?? 18 * 60;
    const now = new Date();
    const ready = isDigestReady(minutesOfDay(now), target);
    digestUnread.set(isDigestUnread(ready, s.digest_last_viewed || "", ymd(now)));
  } catch { digestUnread.set(false); }
}

/** Mark today's digest as read (clears the glow until tomorrow). */
export async function markDigestViewed() {
  digestUnread.set(false);
  try { await setSetting("digest_last_viewed", ymd(new Date())); } catch { /* not in Tauri */ }
}

/** Start the once-a-day digest scheduler (main window only; safe to call without Tauri). */
export function initDigest() {
  if (!browser || started) return;
  started = true;
  setTimeout(check, 3_000);          // shortly after launch
  setInterval(check, 30_000);        // then every 30s
  void refreshDigestUnread();
  setInterval(refreshDigestUnread, 60_000);
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
  await refreshDigestUnread();
}

async function sendDigest() {
  try {
    const [usage, score] = await Promise.all([usageToday(), scoreToday()]);
    const total = usage.reduce((a, u) => a + u.seconds, 0);
    const top = usage[0] ?? null; // usageToday is sorted DESC
    const { title, body } = composeDigest({
      totalSeconds: total, score,
      topName: top?.display_name ?? null, topSeconds: top?.seconds ?? 0,
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
        navIntent.set("digest");
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
