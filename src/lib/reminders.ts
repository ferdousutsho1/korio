import type { Reminder, RepeatRule } from "$lib/api";

/** Unix seconds → the value an `<input type="datetime-local">` expects. */
export function toLocalInput(ts: number): string {
  const d = new Date(ts * 1000);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** `<input type="datetime-local">` value → unix seconds, or null if unparseable. */
export function fromLocalInput(v: string): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})/.exec(v);
  if (!m) return null;
  const d = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]), Number(m[4]), Number(m[5]), 0, 0);
  const ts = Math.floor(d.getTime() / 1000);
  return Number.isFinite(ts) ? ts : null;
}

/** A reminder is due when its time has passed and this particular firing hasn't been surfaced. */
export function isDue(r: Pick<Reminder, "at_ts" | "done" | "fired_at">, nowSecs: number): boolean {
  if (r.done) return false;
  if (r.at_ts > nowSecs) return false;
  return r.fired_at === null || r.fired_at < r.at_ts;
}

/**
 * Next firing time strictly after `after` for a repeating reminder, keeping the
 * original clock time. Returns null for one-off reminders.
 *
 * Steps a real `Date` one calendar day/week at a time so DST shifts don't drift
 * the time of day.
 */
export function nextOccurrence(atTs: number, rule: RepeatRule, after: number): number | null {
  if (rule === "once") return null;
  const d = new Date(atTs * 1000);
  const step = rule === "weekly" ? 7 : 1;
  // Bound the walk so a far-past reminder with a broken clock can't spin forever.
  for (let i = 0; i < 4000; i++) {
    d.setDate(d.getDate() + step);
    const ts = Math.floor(d.getTime() / 1000);
    if (ts <= after) continue;
    if (rule === "weekdays") {
      const wd = d.getDay();
      if (wd === 0 || wd === 6) continue;
    }
    return ts;
  }
  return null;
}

/** Human label for when a reminder fires, relative to today. */
export function formatWhen(ts: number, now = Date.now() / 1000): string {
  const d = new Date(ts * 1000);
  const time = d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
  const dayOf = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime();
  const days = Math.round((dayOf(d) - dayOf(new Date(now * 1000))) / 86_400_000);
  if (days === 0) return `Today ${time}`;
  if (days === 1) return `Tomorrow ${time}`;
  if (days === -1) return `Yesterday ${time}`;
  return `${d.toLocaleDateString(undefined, { month: "short", day: "numeric" })} ${time}`;
}

export const REPEAT_LABELS: { id: RepeatRule; label: string }[] = [
  { id: "once", label: "Once" },
  { id: "daily", label: "Every day" },
  { id: "weekdays", label: "Weekdays" },
  { id: "weekly", label: "Every week" },
];

// ---- Scheduler (main window only; no-ops without Tauri) ----
import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { isMainWindow } from "$lib/sync";
import { listReminders, markReminderFired, snoozeReminder, setReminderDone } from "$lib/api";

/** The reminder currently being shown in the alert overlay, if any. */
export const dueReminder = writable<Reminder | null>(null);
/** Bumped whenever the scheduler changes a reminder, so open views can refresh. */
export const remindersTick = writable(0);

let started = false;

/** Start polling for due reminders (main window only; safe to call without Tauri). */
export function initReminders() {
  if (!browser || started) return;
  started = true;
  setTimeout(check, 4_000);
  setInterval(check, 20_000);
}

async function check() {
  if (!isMainWindow()) return;
  let all: Reminder[];
  try { all = await listReminders(); } catch { return; } // not in Tauri
  const now = Math.floor(Date.now() / 1000);
  const due = all.filter((r) => isDue(r, now)).sort((a, b) => a.at_ts - b.at_ts)[0];
  if (!due) return;
  try { await markReminderFired(due.id, now); } catch { /* surfaced anyway */ }
  dueReminder.set(due);
  remindersTick.update((n) => n + 1);
  notify(due);
}

async function notify(r: Reminder) {
  try {
    const notif = await import("@tauri-apps/plugin-notification");
    let granted = await notif.isPermissionGranted();
    if (!granted) granted = (await notif.requestPermission()) === "granted";
    if (granted) notif.sendNotification({ title: "Korio reminder", body: r.title });
  } catch { /* in-app overlay still shows */ }
}

/** Dismiss a reminder: repeating ones roll to their next occurrence, one-offs complete. */
export async function dismissReminder(r: Reminder) {
  const now = Math.floor(Date.now() / 1000);
  const next = nextOccurrence(r.at_ts, r.repeat_rule, now);
  if (next !== null) await snoozeReminder(r.id, next);
  else await setReminderDone(r.id, true);
  dueReminder.set(null);
  remindersTick.update((n) => n + 1);
}

/** "Remind me later" — push the reminder `minutes` into the future. */
export async function remindLater(r: Reminder, minutes: number) {
  const at = Math.floor(Date.now() / 1000) + Math.max(1, Math.round(minutes)) * 60;
  await snoozeReminder(r.id, at);
  dueReminder.set(null);
  remindersTick.update((n) => n + 1);
}
