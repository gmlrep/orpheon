import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import SegmentedControl from "./SegmentedControl.vue";

const options = [
  { value: "dark", label: "Dark" },
  { value: "light", label: "Light" },
  { value: "system", label: "System" },
];

describe("SegmentedControl", () => {
  it("marks the selected option as pressed", () => {
    const wrapper = mount(SegmentedControl, {
      props: { modelValue: "light", options, label: "Theme" },
    });

    expect(wrapper.get('[role="group"]').attributes("aria-label")).toBe("Theme");
    const buttons = wrapper.findAll("button");
    expect(buttons[1].attributes("aria-pressed")).toBe("true");
    expect(buttons[0].attributes("aria-pressed")).toBe("false");
  });

  it("updates model when a segment is clicked", async () => {
    const wrapper = mount(SegmentedControl, {
      props: {
        modelValue: "dark",
        options,
        "onUpdate:modelValue": (v: string) => wrapper.setProps({ modelValue: v }),
      },
    });

    await wrapper.findAll("button")[2].trigger("click");

    expect(wrapper.props("modelValue")).toBe("system");
    expect(wrapper.findAll("button")[2].attributes("aria-pressed")).toBe("true");
  });
});
