import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { i18n } from "./i18n";
import { router, setupRouterGuards } from "./router";

import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "./styles/tokens.css";
import "./styles/shell.css";

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.use(i18n);
setupRouterGuards();
app.mount("#app");
