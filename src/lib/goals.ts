export interface Goal {
  id: number;
  scope: "kind" | "app" | "total";
  scope_ref: string | null;
  comparator: "gte" | "lte";
  target_seconds: number;
  created_at: number;
}
export interface GoalProgress {
  goal: Goal;
  today_seconds: number;
  met_today: boolean;
  current_streak: number;
  best_streak: number;
}

/** Compact target like "2h" or "30m" (or "1h 30m"). */
export function formatTarget(seconds: number): string {
  const m = Math.round(seconds / 60);
  const h = Math.floor(m / 60);
  const rem = m % 60;
  if (h === 0) return `${rem}m`;
  if (rem === 0) return `${h}h`;
  return `${h}h ${rem}m`;
}

const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** Human label for a goal, e.g. "≥ 2h Productive". `appName` resolves an app-scoped goal's name. */
export function goalLabel(g: Goal, appName?: string): string {
  const cmp = g.comparator === "gte" ? "≥" : "≤";
  const target = formatTarget(g.target_seconds);
  const what =
    g.scope === "total" ? "total" :
    g.scope === "kind" ? cap(g.scope_ref ?? "") :
    (appName ?? "app");
  return `${cmp} ${target} ${what}`;
}
