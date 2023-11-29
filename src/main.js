import { createApp } from "vue";

import Toast from "vue-toastification";
import "vue-toastification/dist/index.css";
import "./assets/styles/app.scss";
import App from "./app.vue";
const app = createApp(App);

import { vuetify } from "./plugins/vuetify";
app.use(vuetify);

import { unifiedApp } from "./plugins/unified/unified-app";
app.use(unifiedApp);

app.use(Toast, {
  transition: "Vue-Toastification__bounce",
  maxToasts: 20,
  newestOnTop: true,
});

app.mount("#app");
