import "vue-toastification/dist/index.css";
import "./assets/styles/app.scss";

import { createApp } from "vue";
import Toast from "vue-toastification";

import App from "./app.vue";
const app = createApp(App);

import { vuetify } from "./plugins/vuetify";
app.use(vuetify);

import { unifiedApp } from "./plugins/unified/unified-app";
app.use(unifiedApp);

import masonry from "vue-next-masonry";
app.use(masonry);



import "viewerjs/dist/viewer.css";

import VueViewer from "v-viewer";
app.use(VueViewer);

app.use(Toast, {
  transition: "Vue-Toastification__bounce",
  maxToasts: 20,
  newestOnTop: true,
});

app.mount("#app");
