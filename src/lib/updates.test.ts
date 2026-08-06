import { describe, it, expect } from "vitest";
import { compareVersions, shouldOfferUpdate, progressFraction, formatNotes,
  DEFAULT_PREFS, type UpdatePrefs } from "./updates";

describe("compareVersions", () => {
  it("orders by numeric segment, not lexically", () => {
    expect(compareVersions("0.10.0", "0.9.0")).toBeGreaterThan(0);
    expect(compareVersions("0.3.0", "0.4.0")).toBeLessThan(0);
    expect(compareVersions("1.0.0", "0.99.99")).toBeGreaterThan(0);
  });
  it("treats missing segments as zero and ignores a leading v", () => {
    expect(compareVersions("1.2", "1.2.0")).toBe(0);
    expect(compareVersions("v0.4.0", "0.4.0")).toBe(0);
    expect(compareVersions("v0.4.1", "0.4.0")).toBeGreaterThan(0);
  });
  it("does not treat junk as newer", () => {
    expect(compareVersions("", "0.3.0")).toBeLessThan(0);
  });
});

const prefs = (p: Partial<UpdatePrefs> = {}): UpdatePrefs => ({ ...DEFAULT_PREFS, ...p });
const NOW = 1_000_000;

describe("shouldOfferUpdate", () => {
  it("offers a strictly newer version", () => {
    expect(shouldOfferUpdate("0.3.0", "0.4.0", prefs(), NOW)).toBe(true);
    expect(shouldOfferUpdate("0.4.0", "0.4.0", prefs(), NOW)).toBe(false);
    expect(shouldOfferUpdate("0.5.0", "0.4.0", prefs(), NOW)).toBe(false);
  });

  it("stays quiet for a skipped version", () => {
    expect(shouldOfferUpdate("0.3.0", "0.4.0", prefs({ skipped: "0.4.0" }), NOW)).toBe(false);
  });

  it("still offers a version newer than the skipped one", () => {
    expect(shouldOfferUpdate("0.3.0", "0.5.0", prefs({ skipped: "0.4.0" }), NOW)).toBe(true);
  });

  it("stays quiet until the remind-later window expires", () => {
    expect(shouldOfferUpdate("0.3.0", "0.4.0", prefs({ remindUntil: NOW + 1 }), NOW)).toBe(false);
    expect(shouldOfferUpdate("0.3.0", "0.4.0", prefs({ remindUntil: NOW - 1 }), NOW)).toBe(true);
  });
});

describe("progressFraction", () => {
  it("clamps to 0..1 and returns null without a known total", () => {
    expect(progressFraction(50, 100)).toBe(0.5);
    expect(progressFraction(150, 100)).toBe(1);
    expect(progressFraction(-5, 100)).toBe(0);
    expect(progressFraction(50, null)).toBeNull();
    expect(progressFraction(50, 0)).toBeNull();
  });
});

describe("formatNotes", () => {
  it("strips markdown and stops at the download footer", () => {
    const body = [
      "## ✨ Added",
      "- **Reminders tab** — schedule `nudges` and [see docs](https://x.test)",
      "",
      "---",
      "## ⬇️ Download",
      "- Portable: grab the zip",
    ].join("\n");
    expect(formatNotes(body)).toEqual([
      "✨ Added",
      "• Reminders tab — schedule nudges and see docs",
    ]);
  });

  it("caps long bodies and handles an empty body", () => {
    const many = Array.from({ length: 40 }, (_, i) => `- line ${i}`).join("\n");
    const out = formatNotes(many, 5);
    expect(out.length).toBe(6);           // 5 lines + the ellipsis marker
    expect(out.at(-1)).toBe("…");
    expect(formatNotes("")).toEqual([]);
  });
});
