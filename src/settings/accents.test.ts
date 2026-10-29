import { describe, expect, it } from "vitest";
import { ACCENT_SWATCHES, accentColor } from "./accents";

describe("accentColor", () => {
  it("returns the swatch at a valid index", () => {
    expect(accentColor(0)).toBe(ACCENT_SWATCHES[0]);
    expect(accentColor(3)).toBe(ACCENT_SWATCHES[3]);
    expect(accentColor(ACCENT_SWATCHES.length - 1)).toBe(
      ACCENT_SWATCHES[ACCENT_SWATCHES.length - 1],
    );
  });

  it("clamps out-of-range indexes", () => {
    expect(accentColor(-5)).toBe(ACCENT_SWATCHES[0]);
    expect(accentColor(99)).toBe(ACCENT_SWATCHES[ACCENT_SWATCHES.length - 1]);
  });
});
