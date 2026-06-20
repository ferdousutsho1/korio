import { describe, it, expect } from "vitest";
import { contrastColor } from "./theme";

describe("contrastColor", () => {
  it("returns white on dark/saturated accents", () => {
    expect(contrastColor("#C2410C")).toBe("#ffffff");
    expect(contrastColor("#000000")).toBe("#ffffff");
    expect(contrastColor("#2F6E4F")).toBe("#ffffff");
  });
  it("returns dark ink on light accents", () => {
    expect(contrastColor("#ffffff")).toBe("#1A1714");
    expect(contrastColor("#FFE600")).toBe("#1A1714");
    expect(contrastColor("#E26A2C")).toBe("#1A1714");
  });
});
