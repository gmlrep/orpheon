import type { Mock } from "vitest";
import type { RouteName } from "../types";

export type MockRouterState = {
  currentRoute: { value: { name: RouteName; path: string } };
  push: Mock<(to: { name: RouteName }) => Promise<void>>;
  reset: () => void;
};

declare global {
  var __orpheonMockRouter: MockRouterState | undefined;
}

/** Same instance installed by `src/test/setup.ts`. */
export function mockRouterState(): MockRouterState {
  const state = globalThis.__orpheonMockRouter;
  if (!state) {
    throw new Error("Router mock not initialized — is setup.ts loaded?");
  }
  return state;
}
