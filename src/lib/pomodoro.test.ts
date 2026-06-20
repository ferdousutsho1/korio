import { describe, it, expect } from "vitest";
import { nextPhase } from "./pomodoro";

describe("nextPhase", () => {
  it("focus → short break, except every Nth focus → long break", () => {
    expect(nextPhase("focus", 1, 4)).toBe("short");
    expect(nextPhase("focus", 2, 4)).toBe("short");
    expect(nextPhase("focus", 3, 4)).toBe("short");
    expect(nextPhase("focus", 4, 4)).toBe("long");
    expect(nextPhase("focus", 8, 4)).toBe("long");
  });
  it("any break → focus", () => {
    expect(nextPhase("short", 4, 4)).toBe("focus");
    expect(nextPhase("long", 4, 4)).toBe("focus");
  });
});
