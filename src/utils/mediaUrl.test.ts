import { describe, expect, it } from "vitest";
import { sanitizeArtworkUrl } from "./mediaUrl";

describe("sanitizeArtworkUrl", () => {
  it("allows http(s) only", () => {
    expect(sanitizeArtworkUrl("https://x/a.png")).toBe("https://x/a.png");
    expect(sanitizeArtworkUrl("http://x/a.png")).toBe("http://x/a.png");
    expect(sanitizeArtworkUrl("data:image/png;base64,xx")).toBeNull();
    expect(sanitizeArtworkUrl("file:///C:/a.png")).toBeNull();
    expect(sanitizeArtworkUrl("C:\\a.png")).toBeNull();
    expect(sanitizeArtworkUrl("")).toBeNull();
  });
});
