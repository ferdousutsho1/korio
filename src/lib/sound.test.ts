// @vitest-environment happy-dom
import { describe, it, expect, beforeEach } from "vitest";
import { SOUNDS, getSoundPref, setSoundPref, type SoundId } from "$lib/sound";
import { getLoopPref, setLoopPref, startLoop, stopAllSounds } from "$lib/sound";

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

describe("sound additions", () => {
  beforeEach(() => localStorage.clear());

  it("catalog includes the new presets", () => {
    const ids = SOUNDS.map((s) => s.id);
    for (const id of ["alarm", "marimba", "pulse", "siren"]) expect(ids).toContain(id);
  });

  it("limit sound defaults to alarm", () => {
    expect(getSoundPref("limit")).toBe("alarm");
  });

  it("loop pref defaults false and round-trips", () => {
    expect(getLoopPref("timer")).toBe(false);
    setLoopPref("timer", true);
    expect(getLoopPref("timer")).toBe(true);
  });

  it("startLoop('silent') returns a no-op stop and stopAllSounds is safe", () => {
    const stop = startLoop("silent");
    expect(typeof stop).toBe("function");
    stop();
    stopAllSounds(); // no throw
  });
});
