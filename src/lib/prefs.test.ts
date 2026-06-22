// @vitest-environment happy-dom
import { describe, it, expect, beforeEach } from "vitest";
import { readJSON, writeJSON } from "$lib/prefs";

describe("prefs", () => {
  beforeEach(() => localStorage.clear());

  it("returns fallback when key absent", () => {
    expect(readJSON("korio.missing", { a: 1 })).toEqual({ a: 1 });
  });

  it("round-trips a value", () => {
    writeJSON("korio.x", { n: 5 });
    expect(readJSON("korio.x", null)).toEqual({ n: 5 });
  });

  it("returns fallback on corrupt JSON", () => {
    localStorage.setItem("korio.bad", "{not json");
    expect(readJSON("korio.bad", "fallback")).toBe("fallback");
  });
});
