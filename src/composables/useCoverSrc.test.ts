import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { resolveMediaSrc, useCoverSrc } from "./useCoverSrc";

describe("useCoverSrc", () => {
  it("returns null when path is missing", () => {
    const path = ref<string | null>(null);
    const src = useCoverSrc(path);
    expect(src.value).toBeNull();
  });

  it("normalizes backslashes and converts via convertFileSrc", () => {
    const path = ref("C:\\Music\\cover.jpg");
    const src = useCoverSrc(path);
    expect(src.value).toBe("asset://localhost/C:/Music/cover.jpg");
  });

  it("passes through remote http(s) URLs", () => {
    expect(resolveMediaSrc("https://cdn.example/art.png")).toBe("https://cdn.example/art.png");
    expect(resolveMediaSrc("http://station/logo.jpg")).toBe("http://station/logo.jpg");
  });
});
