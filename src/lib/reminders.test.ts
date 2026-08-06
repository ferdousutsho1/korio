import { describe, it, expect } from "vitest";
import { toLocalInput, fromLocalInput, isDue, nextOccurrence, formatWhen } from "./reminders";

const at = (y: number, mo: number, d: number, h: number, mi: number) =>
  Math.floor(new Date(y, mo - 1, d, h, mi, 0, 0).getTime() / 1000);

describe("datetime-local round trip", () => {
  it("survives a round trip through the input format", () => {
    const ts = at(2026, 8, 6, 9, 30);
    expect(toLocalInput(ts)).toBe("2026-08-06T09:30");
    expect(fromLocalInput("2026-08-06T09:30")).toBe(ts);
  });

  it("rejects malformed values", () => {
    expect(fromLocalInput("")).toBeNull();
    expect(fromLocalInput("2026-08-06")).toBeNull();
  });
});

describe("isDue", () => {
  const base = { at_ts: 1_000, done: false, fired_at: null as number | null };
  it("fires once the time has passed and it hasn't been surfaced", () => {
    expect(isDue(base, 1_000)).toBe(true);
    expect(isDue(base, 999)).toBe(false);
  });
  it("stays quiet once surfaced for that firing, and after being marked done", () => {
    expect(isDue({ ...base, fired_at: 1_005 }, 2_000)).toBe(false);
    expect(isDue({ ...base, done: true }, 2_000)).toBe(false);
  });
  it("re-fires when the time is pushed past the last firing", () => {
    expect(isDue({ at_ts: 3_000, done: false, fired_at: 1_005 }, 4_000)).toBe(true);
  });
});

describe("nextOccurrence", () => {
  it("returns null for one-off reminders", () => {
    expect(nextOccurrence(at(2026, 8, 6, 9, 0), "once", at(2026, 8, 6, 10, 0))).toBeNull();
  });

  it("keeps the clock time when stepping daily and weekly", () => {
    const start = at(2026, 8, 6, 9, 0); // Thursday
    expect(nextOccurrence(start, "daily", start)).toBe(at(2026, 8, 7, 9, 0));
    expect(nextOccurrence(start, "weekly", start)).toBe(at(2026, 8, 13, 9, 0));
  });

  it("skips the weekend for weekday reminders", () => {
    const fri = at(2026, 8, 7, 9, 0);
    expect(new Date(fri * 1000).getDay()).toBe(5);
    expect(nextOccurrence(fri, "weekdays", fri)).toBe(at(2026, 8, 10, 9, 0)); // Monday
  });

  it("skips forward past a long-missed reminder", () => {
    const start = at(2026, 8, 1, 9, 0);
    const now = at(2026, 8, 20, 12, 0);
    expect(nextOccurrence(start, "daily", now)).toBe(at(2026, 8, 21, 9, 0));
  });
});

describe("formatWhen", () => {
  it("labels today, tomorrow and yesterday relatively", () => {
    const now = at(2026, 8, 6, 12, 0);
    expect(formatWhen(at(2026, 8, 6, 9, 0), now)).toMatch(/^Today /);
    expect(formatWhen(at(2026, 8, 7, 9, 0), now)).toMatch(/^Tomorrow /);
    expect(formatWhen(at(2026, 8, 5, 9, 0), now)).toMatch(/^Yesterday /);
    expect(formatWhen(at(2026, 9, 2, 9, 0), now)).not.toMatch(/Today|Tomorrow|Yesterday/);
  });
});
