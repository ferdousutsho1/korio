import { invoke } from "@tauri-apps/api/core";
import type { Goal, GoalProgress } from "$lib/goals";
export type { Goal, GoalProgress } from "$lib/goals";

export interface App {
  id: number; display_name: string; exe_name: string; kind: string; color: string;
  daily_cap_seconds: number; limit_action: string;
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
export const runningApps = () => invoke<RunningApp[]>("running_apps");
export const removeApp = (id: number) => invoke<void>("remove_app", { id });
export const usageToday = () => invoke<UsageSlice[]>("usage_today");
export const scoreToday = () => invoke<number>("score_today");
export const addApp = (a: { display_name: string; exe_name: string; kind: string; color: string; exe_path?: string | null }) =>
  invoke<number>("add_app", { displayName: a.display_name, exeName: a.exe_name, kind: a.kind, color: a.color, exePath: a.exe_path ?? null });
export const launchApp = (id: number) => invoke<void>("launch_app", { id });

export const usageRange = (from: number, to: number) =>
  invoke<UsageSlice[]>("usage_range", { from, to });
export const dailyTotals = (from: number, to: number) =>
  invoke<DayTotal[]>("daily_totals", { from, to });
export const daySessions = (from: number, to: number) =>
  invoke<SessionRow[]>("day_sessions", { from, to });

export interface LimitEvent {
  exe: string; display_name: string; cap_seconds: number; today_seconds: number;
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
export const listTasks = () => invoke<Task[]>("list_tasks");
export const addTask = (title: string) => invoke<number>("add_task", { title });
export const setTaskDone = (id: number, done: boolean) => invoke<void>("set_task_done", { id, done });
export const updateTaskTitle = (id: number, title: string) => invoke<void>("update_task_title", { id, title });
export const deleteTask = (id: number) => invoke<void>("delete_task", { id });
export const clearDoneTasks = () => invoke<void>("clear_done_tasks");

export interface Note {
  id: number; title: string; body: string; color: string;
  due: string | null; created_at: number; updated_at: number;
}
export const listNotes = () => invoke<Note[]>("list_notes");
export const addNote = () => invoke<number>("add_note");
export const updateNote = (n: { id: number; title: string; body: string; color: string; due: string | null }) =>
  invoke<void>("update_note", { id: n.id, title: n.title, body: n.body, color: n.color, due: n.due });
export const deleteNote = (id: number) => invoke<void>("delete_note", { id });

export type WidgetKind = "stopwatch" | "timer" | "clock" | "pomodoro";
export const openWidget = (kind: WidgetKind) => invoke<void>("open_widget", { kind });
export const closeWidget = (kind: WidgetKind) => invoke<void>("close_widget", { kind });
export const setWidgetAlwaysOnTop = (kind: WidgetKind, on: boolean) =>
  invoke<void>("set_widget_always_on_top", { kind, on });
export const saveWidgetBounds = (kind: WidgetKind, x: number, y: number, w: number, h: number) =>
  invoke<void>("save_widget_bounds", { kind, x, y, w, h });

export const listGoals = () => invoke<Goal[]>("list_goals");
export const goalsProgress = () => invoke<GoalProgress[]>("goals_progress");
export const addGoal = (g: { scope: Goal["scope"]; scopeRef: string | null; comparator: Goal["comparator"]; targetSeconds: number }) =>
  invoke<number>("add_goal", { scope: g.scope, scopeRef: g.scopeRef, comparator: g.comparator, targetSeconds: g.targetSeconds });
export const updateGoal = (id: number, comparator: Goal["comparator"], targetSeconds: number) =>
  invoke<void>("update_goal", { id, comparator, targetSeconds });
export const deleteGoal = (id: number) => invoke<void>("delete_goal", { id });

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
