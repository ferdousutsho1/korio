import { describe, it, expect } from "vitest";
import { isOverdue, NOTE_COLORS } from "./notes";

describe("isOverdue", () => {
  it("is false when there is no due date", () => {
    expect(isOverdue(null, "2026-06-21")).toBe(false);
  });
  it("is true when due is strictly before today", () => {
    expect(isOverdue("2026-06-20", "2026-06-21")).toBe(true);
  });
  it("is false when due is today or later", () => {
    expect(isOverdue("2026-06-21", "2026-06-21")).toBe(false);
    expect(isOverdue("2026-06-22", "2026-06-21")).toBe(false);
  });
});

describe("NOTE_COLORS", () => {
  it("exposes the six palette keys", () => {
    expect(NOTE_COLORS).toEqual(["amber", "rose", "sage", "slate", "butter", "sky"]);
  });
});
