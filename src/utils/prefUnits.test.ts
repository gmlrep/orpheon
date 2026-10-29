import { describe, expect, it } from "vitest";
import { parsePercent, parseSeconds, parseSecondsMs } from "./prefUnits";

describe("prefUnits", () => {
  it("parseSeconds", () => {
    expect(parseSeconds("10 s", 1)).toBe(10);
    expect(parseSeconds("10s", 1)).toBe(10);
    expect(parseSeconds("10seconds", 1)).toBe(10);
    expect(parseSeconds("3 s", 1)).toBe(3);
    expect(parseSeconds("1.5 s", 1)).toBe(1.5);
    expect(parseSeconds("3", 1)).toBe(3);
    expect(parseSeconds("", 7)).toBe(7);
    expect(parseSeconds("nope", 7)).toBe(7);
  });

  it("parseSecondsMs", () => {
    expect(parseSecondsMs("10 s", 0)).toBe(10_000);
    expect(parseSecondsMs(undefined, 5000)).toBe(5000);
  });

  it("parsePercent", () => {
    expect(parsePercent("125%", 100)).toBe(125);
    expect(parsePercent("80", 100)).toBe(80);
    expect(parsePercent("", 50)).toBe(50);
  });
});
