import { invoke } from "@tauri-apps/api/core";

export interface App {
  id: number; display_name: string; exe_name: string; kind: string; color: string;
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

const PALETTE = ["#C2410C", "#7A6F5C", "#B8A98C", "#3A6EA5", "#2F6E4F", "#8A4FB3", "#B23A48"];
export const colorFor = (i: number) => PALETTE[i % PALETTE.length];
