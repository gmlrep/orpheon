import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import GhostButton from "./GhostButton.vue";

describe("GhostButton", () => {
  it("exposes aria-label and slot content", () => {
    const wrapper = mount(GhostButton, {
      props: { label: "Reset" },
      slots: { default: "Reset picture" },
    });

    expect(wrapper.get("button").attributes("aria-label")).toBe("Reset");
    expect(wrapper.text()).toContain("Reset picture");
  });

  it("emits click when pressed", async () => {
    const wrapper = mount(GhostButton, {
      props: { label: "Go" },
      slots: { default: "Go" },
    });

    await wrapper.get("button").trigger("click");

    expect(wrapper.emitted("click")).toHaveLength(1);
  });
});
