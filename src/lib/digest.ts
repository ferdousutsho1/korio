import { formatDuration } from "$lib/format";

/** Local YYYY-MM-DD. */
export function ymd(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/**
 * Local midnight of the day BEFORE `now`'s day.
 *
 * The digest always covers the day that has finished, so it is never built from
 * a day that is still (or barely) in progress — at 00:05 the interesting day is
 * the one that just ended, not the five minutes of the new one.
 */
export function previousDay(now: Date): Date {
  return new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1);
}

/** The day the current digest is about, as YYYY-MM-DD. Rolls over at local midnight. */
export function subjectDay(now: Date): string {
  return ymd(previousDay(now));
}

/**
 * `[from, to)` unix seconds for the local calendar day containing `d`.
 * Built from date components, so it stays correct across DST shifts (a 23- or
 * 25-hour day still spans exactly one calendar day).
 */
export function dayBounds(d: Date): [number, number] {
  const start = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  const end = new Date(d.getFullYear(), d.getMonth(), d.getDate() + 1);
  return [Math.floor(start.getTime() / 1000), Math.floor(end.getTime() / 1000)];
}

/** `[from, to)` unix seconds for the day the digest covers. */
export function subjectDayBounds(now: Date): [number, number] {
  return dayBounds(previousDay(now));
}

/**
 * Send the recap notification when it's enabled and we haven't already sent one
 * for this subject day. Since the subject day rolls at local midnight, this
 * naturally fires just after midnight — or on next launch if Korio was closed.
 */
export function shouldSend(enabled: boolean, lastSent: string, subject: string): boolean {
  return enabled && lastSent !== subject;
}

export interface DigestStats {
  totalSeconds: number;
  score: number;
  topName: string | null;
  topSeconds: number;
}

/** Build the notification title + body from the finished day's stats. */
export function composeDigest(s: DigestStats): { title: string; body: string } {
  const lines = [`${formatDuration(s.totalSeconds)} focused · score ${s.score}`];
  if (s.topName) lines.push(`Top: ${s.topName} ${formatDuration(s.topSeconds)}`);
  return { title: "Korio — yesterday's recap", body: lines.join("\n") };
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

/**
 * Unread = the digest for the day that just ended hasn't been opened yet.
 * `lastViewed` stores the SUBJECT day, not the day it was read on, so opening
 * yesterday's digest at 23:00 doesn't also mark tonight's as read.
 */
export function isDigestUnread(lastViewed: string, subject: string): boolean {
  return lastViewed !== subject;
}

/** Long-form label for the day the digest covers, e.g. "Wednesday, 5 August". */
export function subjectDayLabel(now: Date): string {
  return previousDay(now).toLocaleDateString(undefined,
    { weekday: "long", day: "numeric", month: "long" });
}

// ---- Scheduler (main window only; no-ops without Tauri) ----
import { browser } from "$app/environment";
import { writable } from "svelte/store";
import { isMainWindow } from "$lib/sync";
import { getSettings, setSetting, usageRange, scoreRange } from "$lib/api";
import { navIntent } from "$lib/nav";

let started = false;

/** True while the finished day's digest hasn't been opened — drives the sidebar glow. */
export const digestUnread = writable(false);

/** Recompute the unread flag for the day that just ended. Safe without Tauri. */
export async function refreshDigestUnread() {
  try {
    const s = await getSettings();
    digestUnread.set(isDigestUnread(s.digest_last_viewed || "", subjectDay(new Date())));
  } catch { digestUnread.set(false); }
}

/** Mark the finished day's digest as read (clears the glow until the next midnight). */
export async function markDigestViewed() {
  digestUnread.set(false);
  try { await setSetting("digest_last_viewed", subjectDay(new Date())); } catch { /* not in Tauri */ }
}

/**
 * Watch for the local day rolling over (main window only; safe without Tauri).
 * `check` both refreshes the sidebar glow and sends the recap notification, so
 * a machine left running overnight picks the new digest up within ~30s.
 */
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
  const now = new Date();
  const subject = subjectDay(now);
  // The glow is independent of the notification toggle — the tab is always usable.
  await refreshDigestUnread();
  if (!shouldSend(s.digest_enabled === "true", s.digest_last_sent || "", subject)) return;
  await sendDigest(now);
  await setSetting("digest_last_sent", subject);
}

async function sendDigest(now: Date) {
  try {
    const [from, to] = subjectDayBounds(now);
    const [usage, score] = await Promise.all([usageRange(from, to), scoreRange(from, to)]);
    const total = usage.reduce((a, u) => a + u.seconds, 0);
    const top = usage[0] ?? null; // usage_range is sorted DESC
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
