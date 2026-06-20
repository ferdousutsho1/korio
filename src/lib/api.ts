import { invoke } from "@tauri-apps/api/core";

export interface App {
  id: number; display_name: string; exe_name: string; kind: string; color: string;
  daily_cap_seconds: number; limit_action: string;
}
export interface RunningApp { exe_name: string; title: string; }
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
export const addApp = (a: { display_name: string; exe_name: string; kind: string; color: string }) =>
  invoke<number>("add_app", a);

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

const PALETTE = ["#C2410C", "#7A6F5C", "#B8A98C", "#3A6EA5", "#2F6E4F", "#8A4FB3", "#B23A48"];
export const colorFor = (i: number) => PALETTE[i % PALETTE.length];
