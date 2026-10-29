import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import SeekBar from "./SeekBar.vue";

function mountSeek(modelValue = 0.5) {
  const wrapper = mount(SeekBar, {
    props: {
      modelValue,
      "onUpdate:modelValue": (v: number) => wrapper.setProps({ modelValue: v }),
    },
    attachTo: document.body,
  });
  const el = wrapper.element as HTMLElement;
  el.getBoundingClientRect = () =>
    ({
      left: 0,
      width: 100,
      top: 0,
      height: 28,
      right: 100,
      bottom: 28,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    }) as DOMRect;
  return wrapper;
}

describe("SeekBar", () => {
  it("exposes slider semantics for the current value", () => {
    const wrapper = mountSeek(0.42);
    const slider = wrapper.get('[role="slider"]');
    expect(slider.attributes("aria-valuenow")).toBe("42");
    expect(slider.attributes("aria-valuemin")).toBe("0");
    expect(slider.attributes("aria-valuemax")).toBe("100");
  });

  it("moves with keyboard Home/End and arrows", async () => {
    const wrapper = mountSeek(0.5);
    const slider = wrapper.get('[role="slider"]');

    await slider.trigger("keydown", { key: "Home" });
    expect(wrapper.props("modelValue")).toBe(0);

    await slider.trigger("keydown", { key: "End" });
    expect(wrapper.props("modelValue")).toBe(1);

    await slider.trigger("keydown", { key: "ArrowLeft" });
    expect(wrapper.props("modelValue")).toBeCloseTo(0.99);

    await slider.trigger("keydown", { key: "ArrowRight" });
    expect(wrapper.props("modelValue")).toBe(1);
  });

  it("emits scrubStart and scrubEnd around pointer drag", async () => {
    const wrapper = mountSeek(0.2);
    const slider = wrapper.get('[role="slider"]');

    const capture = vi.fn();
    (slider.element as HTMLElement).setPointerCapture = capture;
    (slider.element as HTMLElement).releasePointerCapture = vi.fn();

    await slider.trigger("pointerdown", { clientX: 40, pointerId: 1 });
    expect(wrapper.emitted("scrubStart")).toHaveLength(1);
    expect(wrapper.props("modelValue")).toBeCloseTo(0.4);

    await slider.trigger("pointermove", { clientX: 70, pointerId: 1 });
    expect(wrapper.props("modelValue")).toBeCloseTo(0.7);

    await slider.trigger("pointerup", { clientX: 70, pointerId: 1 });
    expect(wrapper.emitted("scrubEnd")).toHaveLength(1);
  });
});
