import { describe, it, expect } from "vitest";
import { CARD_CATALOG, DEFAULT_LAYOUT, reconcile, type CardState } from "$lib/dashboard";

describe("dashboard layout reconcile", () => {
  it("returns default layout when nothing saved", () => {
    expect(reconcile(null)).toEqual(DEFAULT_LAYOUT);
  });

  it("preserves saved order and enabled flags for known cards", () => {
    const saved: CardState[] = [
      { id: "focus_score", enabled: false },
      { id: "focus_breakdown", enabled: true },
    ];
    const out = reconcile(saved);
    expect(out[0]).toEqual({ id: "focus_score", enabled: false });
    expect(out[1]).toEqual({ id: "focus_breakdown", enabled: true });
  });

  it("drops unknown ids and appends newly-introduced cards (disabled)", () => {
    const saved: CardState[] = [{ id: "ghost" as any, enabled: true }, { id: "tasks", enabled: true }];
    const out = reconcile(saved);
    expect(out.find((c) => (c.id as string) === "ghost")).toBeUndefined();
    expect(out[0]).toEqual({ id: "tasks", enabled: true });
    // every catalog id present exactly once
    expect(out.map((c) => c.id).sort()).toEqual(CARD_CATALOG.map((c) => c.id).sort());
  });
});
