import { describe, it, expect } from "vitest";
import { formatDuration, formatClock } from "./format";

describe("formatDuration", () => {
  it("formats sub-minute as seconds", () => {
    expect(formatDuration(45)).toBe("45s");
  });
  it("formats minutes and hours compactly", () => {
    expect(formatDuration(60)).toBe("1m");
    expect(formatDuration(3600)).toBe("1h 0m");
    expect(formatDuration(3600 + 48 * 60)).toBe("1h 48m");
  });
  it("clamps negatives to 0s", () => {
    expect(formatDuration(-5)).toBe("0s");
  });
});

describe("formatClock", () => {
  it("renders H:MM:SS for live timers", () => {
    expect(formatClock(0)).toBe("0:00:00");
    expect(formatClock(3600 + 7 * 60 + 9)).toBe("1:07:09");
  });
});
