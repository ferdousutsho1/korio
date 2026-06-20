export const NOTE_COLORS = ["amber", "rose", "sage", "slate", "butter", "sky"] as const;
export type NoteColor = (typeof NOTE_COLORS)[number];

/** Local "YYYY-MM-DD" for today (matches <input type=date> values). */
export function todayYmd(d = new Date()): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/** A note is overdue when it has a due date strictly before `today`. */
export function isOverdue(due: string | null, today: string = todayYmd()): boolean {
  return !!due && due < today;
}
