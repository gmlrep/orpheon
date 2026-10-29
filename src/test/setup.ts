import { vi } from "vitest";
import type { RouteName } from "../types";
import type { MockRouterState } from "./routerMock";

vi.mock("../router", async () => {
  const { shallowRef } = await import("vue");
  const currentRoute = shallowRef({ name: "home" as RouteName, path: "/" });
  const state: MockRouterState = {
    currentRoute,
    push: vi.fn(async (to: { name: RouteName }) => {
      currentRoute.value = {
        name: to.name,
        path: to.name === "home" ? "/" : `/${to.name}`,
      };
    }),
    reset() {
      currentRoute.value = { name: "home", path: "/" };
      state.push.mockClear();
    },
  };
  globalThis.__orpheonMockRouter = state;
  return {
    router: state,
    setupRouterGuards: vi.fn(),
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

Object.defineProperty(window, "matchMedia", {
  writable: true,
  configurable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
