import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import AppSwitch from "./AppSwitch.vue";

describe("AppSwitch", () => {
  it("renders unchecked switch with accessible label", () => {
    const wrapper = mount(AppSwitch, {
      props: { modelValue: false, label: "Hardware acceleration" },
    });

    const sw = wrapper.get('[role="switch"]');
    expect(sw.attributes("aria-checked")).toBe("false");
    expect(sw.attributes("aria-label")).toBe("Hardware acceleration");
  });

  it("toggles model on click", async () => {
    const wrapper = mount(AppSwitch, {
      props: {
        modelValue: false,
        label: "Toggle",
        "onUpdate:modelValue": (v: boolean) => wrapper.setProps({ modelValue: v }),
      },
    });

    await wrapper.get('[role="switch"]').trigger("click");

    expect(wrapper.props("modelValue")).toBe(true);
    expect(wrapper.get('[role="switch"]').attributes("aria-checked")).toBe("true");

    await wrapper.get('[role="switch"]').trigger("click");
    expect(wrapper.props("modelValue")).toBe(false);
  });
});
