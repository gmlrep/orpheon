import { describe, expect, it } from "vitest";
import { formatDuration, initials, toneFor } from "./format";

describe("formatDuration", () => {
  it("returns em dash for nullish, negative, and non-finite", () => {
    expect(formatDuration(null)).toBe("—");
    expect(formatDuration(undefined)).toBe("—");
    expect(formatDuration(-1)).toBe("—");
    expect(formatDuration(Number.NaN)).toBe("—");
    expect(formatDuration(Number.POSITIVE_INFINITY)).toBe("—");
  });

  it("formats under one hour as m:ss", () => {
    expect(formatDuration(0)).toBe("0:00");
    expect(formatDuration(1000)).toBe("0:01");
    expect(formatDuration(65_000)).toBe("1:05");
    expect(formatDuration(3_599_000)).toBe("59:59");
  });

  it("formats one hour and above as h:mm:ss", () => {
    expect(formatDuration(3_600_000)).toBe("1:00:00");
    expect(formatDuration(3_661_000)).toBe("1:01:01");
    expect(formatDuration(12_345_000)).toBe("3:25:45");
  });
});

describe("initials", () => {
  it("takes up to two word initials uppercased", () => {
    expect(initials("Neon Pulse")).toBe("NP");
    expect(initials("alone")).toBe("A");
    expect(initials("one two three")).toBe("OT");
  });

  it("ignores extra whitespace", () => {
    expect(initials("  hello   world  ")).toBe("HW");
    expect(initials("")).toBe("");
  });
});

describe("toneFor", () => {
  it("is stable for the same key", () => {
    expect(toneFor("album-a")).toEqual(toneFor("album-a"));
    expect(toneFor(42)).toEqual(toneFor(42));
  });

  it("returns a palette triple", () => {
    const tone = toneFor("x");
    expect(tone).toHaveLength(3);
    expect(tone[0]).toMatch(/^#/);
  });
});
