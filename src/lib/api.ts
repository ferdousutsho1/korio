import { invoke } from "@tauri-apps/api/core";

export interface App {
  id: number; display_name: string; exe_name: string; kind: string; color: string;
  daily_cap_seconds: number; limit_action: string; category_id: number | null;
}
export interface RunningApp { exe_name: string; title: string; path: string; }
export interface UsageSlice {
  app_id: number; display_name: string; color: string; kind: string; seconds: number;
}
export interface SessionRow {
  display_name: string; color: string; kind: string;
  started_at: number; ended_at: number; active_seconds: number;
}
export interface DayTotal { day: number; seconds: number; }

export const listApps = () => invoke<App[]>("list_apps");

export interface Category { id: number; name: string; color: string; nature: string; created_at: number; }
export interface CategoryUsage { category_id: number | null; name: string; color: string; nature: string; seconds: number; }

export const listCategories = () => invoke<Category[]>("list_categories");
export const addCategory = (name: string, color: string, nature: string) =>
  invoke<number>("add_category", { name, color, nature });
export const updateCategory = (id: number, name: string, color: string, nature: string) =>
  invoke<void>("update_category", { id, name, color, nature });
export const deleteCategory = (id: number) => invoke<void>("delete_category", { id });
export const setAppCategory = (id: number, categoryId: number | null) =>
  invoke<void>("set_app_category", { id, categoryId });
export const usageByCategory = (from: number, to: number) =>
  invoke<CategoryUsage[]>("usage_by_category", { from, to });
export const runningApps = () => invoke<RunningApp[]>("running_apps");
export const removeApp = (id: number) => invoke<void>("remove_app", { id });
export const usageToday = () => invoke<UsageSlice[]>("usage_today");
export const scoreToday = () => invoke<number>("score_today");
export const addApp = (a: { display_name: string; exe_name: string; kind: string; color: string; exe_path?: string | null }) =>
  invoke<number>("add_app", { displayName: a.display_name, exeName: a.exe_name, kind: a.kind, color: a.color, exePath: a.exe_path ?? null });
export const launchApp = (id: number) => invoke<void>("launch_app", { id });
/** Display-only rename; tracking stays keyed on the app's exe name. */
export const renameApp = (id: number, displayName: string) =>
  invoke<void>("rename_app", { id, displayName });

export const usageRange = (from: number, to: number) =>
  invoke<UsageSlice[]>("usage_range", { from, to });
export const scoreRange = (from: number, to: number) =>
  invoke<number>("score_range", { from, to });
export const dailyTotals = (from: number, to: number) =>
  invoke<DayTotal[]>("daily_totals", { from, to });
export const daySessions = (from: number, to: number) =>
  invoke<SessionRow[]>("day_sessions", { from, to });

export interface LimitEvent {
  kind: string; exe: string; display_name: string; cap_seconds: number; today_seconds: number;
  auto_close: boolean;
}
export const setAppLimit = (id: number, daily_cap_seconds: number, limit_action: string) =>
  invoke<void>("set_app_limit", { id, dailyCapSeconds: daily_cap_seconds, limitAction: limit_action });
export const snoozeLimit = (exe: string, minutes: number) =>
  invoke<void>("snooze_limit", { exe, minutes });
export const ignoreLimit = (exe: string) => invoke<void>("ignore_limit", { exe });
export const forceClose = (exe: string) => invoke<void>("force_close", { exe });

export const getSettings = () => invoke<Record<string, string>>("get_settings");
export const setSetting = (key: string, value: string) => invoke<void>("set_setting", { key, value });

export const exportData = (path: string, format: "csv" | "json") =>
  invoke<void>("export_data", { path, format });
export const backupDb = (path: string) => invoke<void>("backup_db", { path });
export const restoreDb = (path: string) => invoke<void>("restore_db", { path });

export const hasPin = () => invoke<boolean>("has_pin");
export const setPin = (pin: string) => invoke<void>("set_pin", { pin });
export const verifyPin = (pin: string) => invoke<boolean>("verify_pin", { pin });
export const clearPin = () => invoke<void>("clear_pin");

const PALETTE = ["#C2410C", "#7A6F5C", "#B8A98C", "#3A6EA5", "#2F6E4F", "#8A4FB3", "#B23A48"];
export const colorFor = (i: number) => PALETTE[i % PALETTE.length];

export interface Task { id: number; title: string; done: boolean; created_at: number; }
export const listTasks = (from?: number, to?: number) =>
  invoke<Task[]>("list_tasks", { from: from ?? null, to: to ?? null });
export const addTask = (title: string, createdAt?: number) =>
  invoke<number>("add_task", { title, createdAt: createdAt ?? null });
export const setTaskDone = (id: number, done: boolean) => invoke<void>("set_task_done", { id, done });
export const updateTaskTitle = (id: number, title: string) => invoke<void>("update_task_title", { id, title });
export const deleteTask = (id: number) => invoke<void>("delete_task", { id });
export const clearDoneTasks = () => invoke<void>("clear_done_tasks");

export interface Note {
  id: number; title: string; body: string; color: string;
  due: string | null; created_at: number; updated_at: number;
  width: number | null; height: number | null;
}
export const listNotes = () => invoke<Note[]>("list_notes");
export const addNote = () => invoke<number>("add_note");
export const updateNote = (n: { id: number; title: string; body: string; color: string; due: string | null }) =>
  invoke<void>("update_note", { id: n.id, title: n.title, body: n.body, color: n.color, due: n.due });
export const deleteNote = (id: number) => invoke<void>("delete_note", { id });
export const setNoteSize = (id: number, width: number, height: number) =>
  invoke<void>("set_note_size", { id, width, height });


export const hideCapture = () => invoke<void>("hide_capture");
export const setCaptureShortcut = (enabled: boolean) => invoke<void>("set_capture_shortcut", { enabled });

export interface BrowserStatus {
  enabled: boolean;
  port: number;
  token: string;
  connected: boolean;
  last_seen_secs: number | null;
  domain: string | null;
}
export const browserStatus = () => invoke<BrowserStatus>("browser_status");
export const setBrowserEnabled = (enabled: boolean) =>
  invoke<void>("set_browser_enabled", { enabled });

export interface SiteUsage { domain: string; seconds: number; }
export const siteUsageToday = () => invoke<SiteUsage[]>("site_usage_today");
export const siteUsageRange = (from: number, to: number) =>
  invoke<SiteUsage[]>("site_usage_range", { from, to });
export const clearSite = (domain: string) => invoke<void>("clear_site", { domain });

export interface SiteCap {
  domain: string; daily_cap_seconds: number; limit_action: string;
  display_name: string | null; category_id: number | null;
}
export const listSiteCaps = () => invoke<SiteCap[]>("list_site_caps");
/** Display-only rename; tracking stays keyed on the domain. Empty clears the override. */
export const setSiteName = (domain: string, displayName: string | null) =>
  invoke<void>("set_site_name", { domain, displayName });
export const setSiteCategory = (domain: string, categoryId: number | null) =>
  invoke<void>("set_site_category", { domain, categoryId });

export interface SiteSlice {
  domain: string; display_name: string | null; category_id: number | null;
  color: string; seconds: number;
}
export const siteSlices = (from: number, to: number) =>
  invoke<SiteSlice[]>("site_slices", { from, to });
export const setSiteLimit = (domain: string, daily_cap_seconds: number, limit_action: string) =>
  invoke<void>("set_site_limit", { domain, dailyCapSeconds: daily_cap_seconds, limitAction: limit_action });
export const snoozeSiteLimit = (domain: string, minutes: number) =>
  invoke<void>("snooze_site_limit", { domain, minutes });
export const ignoreSiteLimit = (domain: string) => invoke<void>("ignore_site_limit", { domain });
export const blockSite = (domain: string) => invoke<void>("block_site", { domain });

export const clearAlertTopmost = () => invoke<void>("clear_alert_topmost");

export type RepeatRule = "once" | "daily" | "weekdays" | "weekly";
export interface Reminder {
  id: number; title: string; at_ts: number; repeat_rule: RepeatRule;
  done: boolean; fired_at: number | null; created_at: number;
}
export const listReminders = () => invoke<Reminder[]>("list_reminders");
export const addReminder = (title: string, atTs: number, repeatRule: RepeatRule) =>
  invoke<number>("add_reminder", { title, atTs, repeatRule });
export const updateReminder = (id: number, title: string, atTs: number, repeatRule: RepeatRule) =>
  invoke<void>("update_reminder", { id, title, atTs, repeatRule });
export const setReminderDone = (id: number, done: boolean) =>
  invoke<void>("set_reminder_done", { id, done });
export const snoozeReminder = (id: number, atTs: number) =>
  invoke<void>("snooze_reminder", { id, atTs });
export const markReminderFired = (id: number, firedAt: number) =>
  invoke<void>("mark_reminder_fired", { id, firedAt });
export const deleteReminder = (id: number) => invoke<void>("delete_reminder", { id });
