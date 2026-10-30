import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import AboutPopup from "./popups/AboutPopup.vue";
import ShortcutsPopup from "./popups/ShortcutsPopup.vue";
import CustomizePopup from "./popups/CustomizePopup.vue";
import ContextMenuPopup from "./popups/ContextMenuPopup.vue";
import { i18n } from "./i18n";
import { router, setupRouterGuards } from "./router";

import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "./styles/tokens.css";
import "./styles/shell.css";

const popup = new URLSearchParams(window.location.search).get("popup");

const popupRoot =
  popup === "about"
    ? AboutPopup
    : popup === "shortcuts"
      ? ShortcutsPopup
      : popup === "customize"
        ? CustomizePopup
        : popup === "context"
          ? ContextMenuPopup
          : null;

if (popupRoot) {
  document.documentElement.dataset.popup = popup ?? "";
  const app = createApp(popupRoot);
  app.use(createPinia());
  app.use(i18n);
  app.mount("#app");
} else {
  const app = createApp(App);
  app.use(createPinia());
  app.use(router);
  app.use(i18n);
  setupRouterGuards();
  app.mount("#app");
}
