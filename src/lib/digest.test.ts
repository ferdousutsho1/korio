import { describe, it, expect } from "vitest";
import { ymd, shouldSend, composeDigest, digestQuips, isDigestUnread, limitPinRequired,
  previousDay, subjectDay, dayBounds, subjectDayBounds, type Highlights } from "./digest";

describe("ymd", () => {
  it("is the zero-padded local date", () => {
    expect(ymd(new Date(2026, 0, 3))).toBe("2026-01-03");
  });
});

describe("the day a digest is about", () => {
  it("is always the day BEFORE now — the one that has actually finished", () => {
    // The reported bug: at 00:00 the digest covered the brand-new (empty) day.
    expect(subjectDay(new Date(2026, 7, 4, 0, 0, 0))).toBe("2026-08-03");
    expect(subjectDay(new Date(2026, 7, 4, 23, 59, 0))).toBe("2026-08-03");
    // ...and it rolls over exactly at midnight, not at some configured hour.
    expect(subjectDay(new Date(2026, 7, 5, 0, 0, 1))).toBe("2026-08-04");
  });

  it("steps back across month and year boundaries", () => {
    expect(subjectDay(new Date(2026, 7, 1, 9, 0))).toBe("2026-07-31");
    expect(subjectDay(new Date(2026, 0, 1, 9, 0))).toBe("2025-12-31");
  });

  it("returns local midnight of that day", () => {
    const d = previousDay(new Date(2026, 7, 4, 14, 37, 12));
    expect([d.getFullYear(), d.getMonth(), d.getDate()]).toEqual([2026, 7, 3]);
    expect([d.getHours(), d.getMinutes(), d.getSeconds()]).toEqual([0, 0, 0]);
  });
});

describe("dayBounds", () => {
  it("spans exactly one calendar day, half-open", () => {
    const [from, to] = dayBounds(new Date(2026, 7, 4, 13, 0));
    expect(from).toBe(Math.floor(new Date(2026, 7, 4).getTime() / 1000));
    expect(to).toBe(Math.floor(new Date(2026, 7, 5).getTime() / 1000));
  });

  it("wraps to the next month correctly", () => {
    const [, to] = dayBounds(new Date(2026, 7, 31, 13, 0));
    expect(to).toBe(Math.floor(new Date(2026, 8, 1).getTime() / 1000));
  });

  it("subjectDayBounds covers the finished day, not the current one", () => {
    const now = new Date(2026, 7, 4, 0, 5, 0);
    const [from, to] = subjectDayBounds(now);
    expect(from).toBe(Math.floor(new Date(2026, 7, 3).getTime() / 1000));
    expect(to).toBe(Math.floor(new Date(2026, 7, 4).getTime() / 1000));
    expect(to).toBeLessThanOrEqual(Math.floor(now.getTime() / 1000));
  });
});

describe("shouldSend", () => {
  it("fires once per subject day when enabled", () => {
    expect(shouldSend(true, "2026-08-02", "2026-08-03")).toBe(true);
    expect(shouldSend(true, "", "2026-08-03")).toBe(true);
  });
  it("does not fire when disabled", () => {
    expect(shouldSend(false, "", "2026-08-03")).toBe(false);
  });
  it("does not fire twice for the same day", () => {
    expect(shouldSend(true, "2026-08-03", "2026-08-03")).toBe(false);
  });
});

describe("composeDigest", () => {
  it("includes time, score and the top app", () => {
    const { title, body } = composeDigest({ totalSeconds: 12000, score: 78, topName: "VS Code", topSeconds: 7800 });
    expect(title).toBe("Korio — yesterday's recap");
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
  it("glows until the finished day's digest has been opened", () => {
    expect(isDigestUnread("2026-08-02", "2026-08-03")).toBe(true);
    expect(isDigestUnread("2026-08-03", "2026-08-03")).toBe(false);
    expect(isDigestUnread("", "2026-08-03")).toBe(true);
  });

  it("keys on the SUBJECT day, so reading late doesn't pre-read tomorrow's", () => {
    // Read the 3rd's digest at 23:00 on the 4th → still unread once the 4th ends.
    const viewed = subjectDay(new Date(2026, 7, 4, 23, 0));
    expect(isDigestUnread(viewed, subjectDay(new Date(2026, 7, 4, 23, 30)))).toBe(false);
    expect(isDigestUnread(viewed, subjectDay(new Date(2026, 7, 5, 0, 1)))).toBe(true);
  });
});
