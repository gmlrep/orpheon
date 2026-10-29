import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useUiStore } from "./ui";
import { mockRouterState } from "../test/routerMock";
import { mockMatchMedia } from "../test/helpers";

describe("ui store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockRouterState().reset();
    mockMatchMedia(() => false);
  });

  it("cycles sidebar open → rail → hidden on wide screens", () => {
    const store = useUiStore();
    expect(store.sidebarMode).toBe("open");

    store.cycleSidebar();
    expect(store.sidebarMode).toBe("rail");

    store.cycleSidebar();
    expect(store.sidebarMode).toBe("hidden");

    store.cycleSidebar();
    expect(store.sidebarMode).toBe("open");
  });

  it("toggles sheet ↔ hidden on narrow screens", () => {
    mockMatchMedia((q) => q.includes("max-width: 900px"));
    const store = useUiStore();
    store.sidebarMode = "sheet";

    store.cycleSidebar();
    expect(store.sidebarMode).toBe("hidden");

    store.cycleSidebar();
    expect(store.sidebarMode).toBe("sheet");
  });

  it("setFullscreen hides sidebar and restores previous mode", () => {
    const store = useUiStore();
    store.sidebarMode = "open";

    store.setFullscreen(true);
    expect(store.fullscreen).toBe(true);
    expect(store.sidebarMode).toBe("hidden");
    expect(store.uiHidden).toBe(false);

    store.setFullscreen(false);
    expect(store.fullscreen).toBe(false);
    expect(store.sidebarMode).toBe("open");
  });

  it("setFullscreen restores hidden as rail", () => {
    const store = useUiStore();
    store.sidebarMode = "hidden";

    store.setFullscreen(true);
    store.setFullscreen(false);

    expect(store.sidebarMode).toBe("rail");
  });

  it("navigate to player collapses open sidebar to rail", async () => {
    const store = useUiStore();
    store.sidebarMode = "open";

    store.navigate("player");
    await Promise.resolve();

    expect(store.sidebarMode).toBe("rail");
    expect(mockRouterState().push).toHaveBeenCalledWith({ name: "player" });
    expect(mockRouterState().currentRoute.value.name).toBe("player");
  });

  it("navigate does not push when already on route", () => {
    const store = useUiStore();
    mockRouterState().currentRoute.value = { name: "home", path: "/" };

    store.navigate("home");

    expect(mockRouterState().push).not.toHaveBeenCalled();
  });

  it("route follows the current router name", () => {
    const store = useUiStore();
    expect(store.route).toBe("home");

    mockRouterState().currentRoute.value = { name: "eq", path: "/eq" };
    expect(store.route).toBe("eq");
  });

  it("toggleUiHidden flips visibility flag", () => {
    const store = useUiStore();
    expect(store.uiHidden).toBe(false);
    store.toggleUiHidden();
    expect(store.uiHidden).toBe(true);
  });
});
