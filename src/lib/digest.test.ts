import { describe, it, expect } from "vitest";
import { minutesOfDay, parseHm, ymd, shouldSend, composeDigest,
  digestQuips, isDigestReady, isDigestUnread, limitPinRequired, type Highlights } from "./digest";

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
  it("includes time, score and the top app", () => {
    const { title, body } = composeDigest({ totalSeconds: 12000, score: 78, topName: "VS Code", topSeconds: 7800 });
    expect(title).toBe("Korio — today's recap");
    expect(body).toContain("3h 20m focused");
    expect(body).toContain("score 78");
    expect(body).toContain("Top: VS Code 2h 10m");
  });
  it("omits the top line when there was no activity", () => {
    const { body } = composeDigest({ totalSeconds: 0, score: 0, topName: null, topSeconds: 0 });
    expect(body).toContain("0s focused");
    expect(body).not.toContain("Top:");
  });
});

const base: Highlights = {
  totalSeconds: 7200, score: 60, topName: "VS Code", topSeconds: 5000,
  topSiteName: null, topSiteSeconds: 0, tasksDone: 0, tasksTotal: 0,
  pomodoros: 0, overages: [],
};

describe("digestQuips", () => {
  it("calls out the worst overage first and scales the tone with how far over it went", () => {
    const mild = digestQuips({ ...base, overages: [{ name: "Reddit", kind: "site", capSeconds: 600, usedSeconds: 900 }] });
    expect(mild[0]).toContain("Reddit");
    expect(mild[0]).toContain("Close, though");

    const wild = digestQuips({ ...base, overages: [{ name: "Reddit", kind: "site", capSeconds: 600, usedSeconds: 3600 }] });
    expect(wild[0]).toContain("just a suggestion");
  });

  it("picks the biggest overage out of several and notes the count", () => {
    const q = digestQuips({ ...base, overages: [
      { name: "Slack", kind: "app", capSeconds: 600, usedSeconds: 700 },
      { name: "YouTube", kind: "site", capSeconds: 600, usedSeconds: 1400 },
    ] });
    expect(q[0]).toContain("YouTube");
    expect(q[1]).toContain("2 limits ignored");
  });

  it("covers the quiet day, the great day and the to-do sweep", () => {
    expect(digestQuips({ ...base, totalSeconds: 0 })[0]).toContain("Nothing tracked");
    expect(digestQuips({ ...base, score: 90 })[0]).toContain("Suspiciously productive");
    expect(digestQuips({ ...base, tasksDone: 3, tasksTotal: 3 })).toContain("Every to-do checked off. Show-off.");
  });

  it("always says something, and never more than three lines", () => {
    expect(digestQuips(base).length).toBeGreaterThan(0);
    const noisy = digestQuips({
      ...base, score: 10, tasksDone: 0, tasksTotal: 4,
      overages: [
        { name: "A", kind: "app", capSeconds: 60, usedSeconds: 600 },
        { name: "B", kind: "site", capSeconds: 60, usedSeconds: 120 },
      ],
    });
    expect(noisy.length).toBe(3);
  });
});

describe("limitPinRequired", () => {
  it("needs both the setting and a stored PIN", () => {
    expect(limitPinRequired({ limit_pin_enabled: "true" }, true)).toBe(true);
    expect(limitPinRequired({ limit_pin_enabled: "true" }, false)).toBe(false);
    expect(limitPinRequired({ limit_pin_enabled: "false" }, true)).toBe(false);
    expect(limitPinRequired({}, true)).toBe(false);
  });
});

describe("digest unread state", () => {
  it("is ready once the clock passes the configured time", () => {
    expect(isDigestReady(1080, 1080)).toBe(true);
    expect(isDigestReady(1079, 1080)).toBe(false);
  });
  it("glows only for a ready digest that today hasn't opened", () => {
    expect(isDigestUnread(true, "2026-06-20", "2026-06-21")).toBe(true);
    expect(isDigestUnread(true, "2026-06-21", "2026-06-21")).toBe(false);
    expect(isDigestUnread(false, "", "2026-06-21")).toBe(false);
  });
});
