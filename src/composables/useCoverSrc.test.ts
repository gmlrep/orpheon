import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { DEFAULT_COVER_SRC } from "../assets/defaultCover";
import { resolveCoverSrc, resolveMediaSrc, useCoverSrc } from "./useCoverSrc";

describe("useCoverSrc", () => {
  it("falls back to default cover when path is missing", () => {
    const path = ref<string | null>(null);
    const src = useCoverSrc(path);
    expect(src.value).toBe(DEFAULT_COVER_SRC);
    expect(resolveCoverSrc(null)).toBe(DEFAULT_COVER_SRC);
    expect(resolveMediaSrc(null)).toBeNull();
  });

  it("normalizes backslashes and converts via convertFileSrc", () => {
    const path = ref("C:\\Music\\cover.jpg");
    const src = useCoverSrc(path);
    expect(src.value).toBe("asset://localhost/C:/Music/cover.jpg");
  });

  it("passes through remote http(s) URLs only", () => {
    expect(resolveMediaSrc("https://cdn.example/art.png")).toBe("https://cdn.example/art.png");
    expect(resolveMediaSrc("http://station/logo.jpg")).toBe("http://station/logo.jpg");
    expect(resolveMediaSrc("data:image/png;base64,xx")).toBeNull();
    expect(resolveMediaSrc("file:///C:/a.png")).toBeNull();
    expect(resolveCoverSrc("data:image/png;base64,xx")).toBe(DEFAULT_COVER_SRC);
  });
});
