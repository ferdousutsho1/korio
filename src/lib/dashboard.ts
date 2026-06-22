import { readJSON, writeJSON } from "$lib/prefs";

export type CardId =
  | "focus_breakdown" | "focus_score" | "tracking_now"
  | "today_stats" | "tasks" | "notes" | "goals";

export interface CardState { id: CardId; enabled: boolean; }

export const CARD_CATALOG: { id: CardId; title: string }[] = [
  { id: "focus_breakdown", title: "Focus breakdown" },
  { id: "focus_score", title: "Focus score" },
  { id: "tracking_now", title: "Tracking now" },
  { id: "today_stats", title: "Today's stats" },
  { id: "tasks", title: "Tasks" },
  { id: "notes", title: "Notes" },
  { id: "goals", title: "Goals" },
];

export const DEFAULT_LAYOUT: CardState[] = [
  { id: "focus_breakdown", enabled: true },
  { id: "focus_score", enabled: true },
  { id: "tracking_now", enabled: true },
  { id: "today_stats", enabled: false },
  { id: "tasks", enabled: false },
  { id: "notes", enabled: false },
  { id: "goals", enabled: false },
];

const KEY = "korio.dashboard.layout";
const known = (id: string): id is CardId => CARD_CATALOG.some((c) => c.id === id);

/** Keep saved order for known cards, drop unknowns, append missing catalog cards disabled. */
export function reconcile(saved: CardState[] | null): CardState[] {
  if (!saved || !Array.isArray(saved)) return DEFAULT_LAYOUT.map((c) => ({ ...c }));
  const seen = new Set<CardId>();
  const out: CardState[] = [];
  for (const c of saved) {
    if (c && known(c.id) && !seen.has(c.id)) {
      out.push({ id: c.id, enabled: !!c.enabled });
      seen.add(c.id);
    }
  }
  for (const c of CARD_CATALOG) {
    if (!seen.has(c.id)) out.push({ id: c.id, enabled: false });
  }
  return out;
}

export function loadLayout(): CardState[] { return reconcile(readJSON<CardState[] | null>(KEY, null)); }
export function saveLayout(layout: CardState[]): void { writeJSON(KEY, layout); }
