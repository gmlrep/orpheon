import { createRouter, createWebHistory } from "vue-router";
import type { RouteName } from "../types";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home" satisfies RouteName,
      component: () => import("../views/HomeView.vue"),
    },
    {
      path: "/videos",
      name: "videos" satisfies RouteName,
      component: () => import("../views/LibraryView.vue"),
      props: { kind: "video" },
    },
    {
      path: "/music",
      name: "music" satisfies RouteName,
      component: () => import("../views/LibraryView.vue"),
      props: { kind: "audio" },
    },
    {
      path: "/playlists",
      name: "playlists" satisfies RouteName,
      component: () => import("../views/PlaylistsView.vue"),
    },
    {
      path: "/recent",
      name: "recent" satisfies RouteName,
      component: () => import("../views/RecentView.vue"),
    },
    {
      path: "/stream",
      name: "stream" satisfies RouteName,
      component: () => import("../views/StreamView.vue"),
    },
    {
      path: "/settings",
      name: "settings" satisfies RouteName,
      component: () => import("../views/SettingsView.vue"),
    },
    {
      path: "/eq",
      name: "eq" satisfies RouteName,
      component: () => import("../views/EqView.vue"),
    },
    {
      path: "/player",
      name: "player" satisfies RouteName,
      component: () => import("../views/PlayerView.vue"),
    },
    { path: "/:pathMatch(.*)*", redirect: { name: "home" } },
  ],
});

/** Wire Pinia-dependent navigation side effects after the app store is ready. */
export function setupRouterGuards() {
  router.afterEach((to, from) => {
    void import("../stores/playback").then(({ usePlaybackStore }) => {
      const playback = usePlaybackStore();
      if (to.name !== "player") {
        playback.closeOverlays();
      }
      if (from.name === "player" && to.name !== "player") {
        void playback.tearDownVideoSurface().catch(() => {
          /* ignore */
        });
      }
    });
  });
}
