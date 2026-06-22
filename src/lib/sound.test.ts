// @vitest-environment happy-dom
import { describe, it, expect, beforeEach } from "vitest";
import { SOUNDS, getSoundPref, setSoundPref, type SoundId } from "$lib/sound";

describe("sound prefs", () => {
  beforeEach(() => localStorage.clear());

  it("defaults pomodoro and timer to a real sound", () => {
    expect(getSoundPref("pomodoro")).toBe("chime");
    expect(getSoundPref("timer")).toBe("beep");
  });

  it("round-trips a chosen sound", () => {
    setSoundPref("timer", "bell");
    expect(getSoundPref("timer")).toBe("bell");
  });

  it("falls back to default for an unknown stored value", () => {
    localStorage.setItem("korio.sound.timer", JSON.stringify("nonsense"));
    expect(getSoundPref("timer")).toBe("beep");
  });

  it("catalog includes a silent option", () => {
    expect(SOUNDS.map((s: { id: SoundId }) => s.id)).toContain("silent");
  });
});
