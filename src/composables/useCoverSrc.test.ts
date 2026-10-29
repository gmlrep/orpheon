import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { useCoverSrc } from "./useCoverSrc";

describe("useCoverSrc", () => {
  it("returns null when path is missing", () => {
    const path = ref<string | null>(null);
    const src = useCoverSrc(path);
    expect(src.value).toBeNull();

    path.value = undefined as unknown as null;
    expect(src.value).toBeNull();
  });

  it("normalizes backslashes and converts via convertFileSrc", () => {
    const path = ref("C:\\Music\\cover.jpg");
    const src = useCoverSrc(path);
    expect(src.value).toBe("asset://localhost/C:/Music/cover.jpg");
  });
});
