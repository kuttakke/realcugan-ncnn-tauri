import "@mdi/font/css/materialdesignicons.min.css";
import "vuetify/styles";

import { createVuetify } from "vuetify";

const draculaDark = {
  dark: true,
  colors: {
    background: "#414558",
    surface: "#282a36",
    primary: "#ff79c6",
    secondary: "#bd93f9",
    error: "#ff5555",
  },
};


const nord = {
  dark: false,
  colors: {
    background: "#ECEFF4",
    surface: "#282a36",
    primary: "#ffffff",
    secondary: "#81A1C1",
    error: "#BF616A",
  },
};

export const vuetify = createVuetify({
  theme: {
    defaultTheme: "nord",
    themes: {
      draculaDark,
      nord,
    },
  },
});
