import { describe, it, expect } from "vitest";
import { presetRange, heatLevel, dayStartLocal, eachDay } from "./ranges";

const NOW = new Date(2026, 5, 15, 12, 0, 0); // 2026-06-15 12:00 local

describe("presetRange", () => {
  it("today spans local midnight to +24h", () => {
    const { from, to } = presetRange("today", NOW);
    expect(to - from).toBe(86400);
    expect(from).toBe(Math.floor(new Date(2026, 5, 15, 0, 0, 0).getTime() / 1000));
  });
  it("7d spans 7 local days", () => {
    const { from, to } = presetRange("7d", NOW);
    expect(to - from).toBe(7 * 86400);
  });
});

describe("heatLevel", () => {
  it("is 0 for none and 1..4 by proportion of max", () => {
    expect(heatLevel(0, 100)).toBe(0);
    expect(heatLevel(100, 100)).toBe(4);
    expect(heatLevel(1, 100)).toBe(1);
    expect(heatLevel(0, 0)).toBe(0);
  });
});

describe("eachDay / dayStartLocal", () => {
  it("eachDay lists local day-starts (calendar increments)", () => {
    const from = dayStartLocal(NOW);
    const days = eachDay(from, from + 3 * 86400);
    expect(days.length).toBe(3);
    expect(days[0]).toBe(from);
    expect(days[1]).toBe(dayStartLocal(new Date((from + 86400) * 1000)));
  });
});
