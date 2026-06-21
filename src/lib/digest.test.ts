import { describe, it, expect } from "vitest";
import { minutesOfDay, parseHm, ymd, shouldSend, composeDigest } from "./digest";

describe("parseHm", () => {
  it("parses HH:MM to minutes", () => {
    expect(parseHm("18:00")).toBe(1080);
    expect(parseHm("00:00")).toBe(0);
    expect(parseHm("09:30")).toBe(570);
  });
  it("returns null for garbage", () => {
    expect(parseHm("nope")).toBeNull();
    expect(parseHm("25:00")).toBeNull();
    expect(parseHm("")).toBeNull();
  });
});

describe("minutesOfDay / ymd", () => {
  it("minutesOfDay uses local hours+minutes", () => {
    expect(minutesOfDay(new Date(2026, 5, 21, 18, 5))).toBe(18 * 60 + 5);
  });
  it("ymd is zero-padded local date", () => {
    expect(ymd(new Date(2026, 0, 3))).toBe("2026-01-03");
  });
});

describe("shouldSend", () => {
  it("fires when enabled, now >= target, and not sent today", () => {
    expect(shouldSend(true, 1080, 1080, "2026-06-20", "2026-06-21")).toBe(true);
    expect(shouldSend(true, 1100, 1080, "", "2026-06-21")).toBe(true);
  });
  it("does not fire when disabled", () => {
    expect(shouldSend(false, 1100, 1080, "", "2026-06-21")).toBe(false);
  });
  it("does not fire before the target time", () => {
    expect(shouldSend(true, 1000, 1080, "", "2026-06-21")).toBe(false);
  });
  it("does not fire twice the same day", () => {
    expect(shouldSend(true, 1100, 1080, "2026-06-21", "2026-06-21")).toBe(false);
  });
});

describe("composeDigest", () => {
  it("includes time, score, top app, and goals", () => {
    const { title, body } = composeDigest({ totalSeconds: 12000, score: 78, topName: "VS Code", topSeconds: 7800, goalsMet: 2, goalsTotal: 3 });
    expect(title).toBe("Korio — today's recap");
    expect(body).toContain("3h 20m focused");
    expect(body).toContain("score 78");
    expect(body).toContain("Top: VS Code 2h 10m");
    expect(body).toContain("2/3 goals met");
  });
  it("omits the top line when there was no activity and the goals line when there are no goals", () => {
    const { body } = composeDigest({ totalSeconds: 0, score: 0, topName: null, topSeconds: 0, goalsMet: 0, goalsTotal: 0 });
    expect(body).toContain("0s focused");
    expect(body).not.toContain("Top:");
    expect(body).not.toContain("goals met");
  });
});
