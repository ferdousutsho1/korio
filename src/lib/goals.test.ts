import { describe, it, expect } from "vitest";
import { goalLabel, formatTarget, type Goal } from "./goals";

const base: Goal = { id: 1, scope: "total", scope_ref: null, comparator: "gte", target_seconds: 7200, created_at: 0 };

describe("goalLabel", () => {
  it("labels a total goal in hours", () => {
    expect(goalLabel(base)).toBe("≥ 2h total");
  });
  it("labels a kind goal with the capitalized kind", () => {
    expect(goalLabel({ ...base, scope: "kind", scope_ref: "productive", comparator: "gte", target_seconds: 7200 }))
      .toBe("≥ 2h Productive");
  });
  it("labels a ≤ limit in minutes", () => {
    expect(goalLabel({ ...base, scope: "kind", scope_ref: "distracting", comparator: "lte", target_seconds: 1800 }))
      .toBe("≤ 30m Distracting");
  });
});

describe("formatTarget", () => {
  it("formats whole hours, whole minutes, and mixed", () => {
    expect(formatTarget(7200)).toBe("2h");
    expect(formatTarget(1800)).toBe("30m");
    expect(formatTarget(5400)).toBe("1h 30m");
  });
});
