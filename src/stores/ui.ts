import { defineStore } from "pinia";
import { computed, shallowRef } from "vue";
import type { RouteName, SidebarMode } from "../types";
import { router } from "../router";
import { NARROW_LAYOUT_MQ } from "../config/app";

export const useUiStore = defineStore("ui", () => {
  const sidebarMode = shallowRef<SidebarMode>("open");
  const settingsCat = shallowRef("general");
  const fullscreen = shallowRef(false);
  const uiHidden = shallowRef(false);
  /** Sidebar mode to restore after leaving fullscreen. */
  let sidebarBeforeFullscreen: SidebarMode | null = null;

  const route = computed(() => (router.currentRoute.value.name as RouteName) || "home");

  const sideToggleState = computed(() => {
    if (sidebarMode.value === "open" || sidebarMode.value === "sheet") return "max";
    if (sidebarMode.value === "rail") return "min";
    return "none";
  });

  function cycleSidebar() {
    const narrow = window.matchMedia(NARROW_LAYOUT_MQ).matches;
    if (narrow) {
      sidebarMode.value = sidebarMode.value === "sheet" ? "hidden" : "sheet";
      return;
    }
    const order: SidebarMode[] = ["open", "rail", "hidden"];
    const i = order.indexOf(sidebarMode.value);
    sidebarMode.value = order[(i + 1) % order.length];
  }

  /** Navigate without changing the user's chosen sidebar mode. */
  function navigate(name: RouteName) {
    if (name === "player" && route.value !== "player" && sidebarMode.value === "open") {
      sidebarMode.value = "rail";
    }
    if (route.value !== name) {
      void router.push({ name });
    }
  }

  function setSettingsCat(value: string) {
    settingsCat.value = value;
  }

  function setUiHidden(value: boolean) {
    uiHidden.value = value;
  }

  function toggleUiHidden() {
    uiHidden.value = !uiHidden.value;
  }

  function setFullscreen(value: boolean) {
    if (value === fullscreen.value) return;
    fullscreen.value = value;
    if (value) {
      sidebarBeforeFullscreen = sidebarMode.value;
      sidebarMode.value = "hidden";
      uiHidden.value = false;
    } else {
      uiHidden.value = false;
      const restore = sidebarBeforeFullscreen ?? "rail";
      sidebarBeforeFullscreen = null;
      sidebarMode.value = restore === "hidden" ? "rail" : restore;
    }
  }

  return {
    sidebarMode,
    route,
    settingsCat,
    fullscreen,
    uiHidden,
    sideToggleState,
    cycleSidebar,
    navigate,
    setSettingsCat,
    setUiHidden,
    toggleUiHidden,
    setFullscreen,
  };
});
