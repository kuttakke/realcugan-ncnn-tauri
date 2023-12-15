<script setup>


import { appWindow } from "@tauri-apps/api/window";
import { useStorage } from "@vueuse/core";
import { onMounted } from "vue";
import { useTheme } from "vuetify";

import Real from "./modules/real/mod.vue";
import { isDark, toggleDark } from "./plugins/dark";

const theme = useTheme();
const myThemes = ["draculaDark", "nord"];

const nowTheme = useStorage("theme", isDark.value ? "dark" : "light");
if (nowTheme.value.includes("dark") || nowTheme.value.includes("Dark")) {
  theme.global.name.value = myThemes[0];
} else {
  theme.global.name.value = myThemes[1];
}
 // 阻止默认事件Function
const forbidDefaultEvent = (e) => {
    e.preventDefault();
  }
const setTheme = () => {
  if (isDark.value) {
    theme.global.name.value = myThemes[0];
  } else {
    theme.global.name.value = myThemes[1];
  }
};
setTheme();
const changeTheme = () => {
  toggleDark();
  setTheme();
};

const changeChildAttr = (parent) => {
  if (parent.getAttributeNames().includes("none-drag-region")) {
    return;
  }

  for (let i = 0; i < parent.children.length; i++) {
    let child = parent.children[i];
    changeChildAttr(child);
  }
  parent.setAttribute("data-tauri-drag-region", "");
};

const setDragAttr = () => {
  const dragRegion = document.querySelector("[data-tauri-drag-region]");
  changeChildAttr(dragRegion);
};
onMounted(() => {
  setDragAttr()
  // NOTE - 阻止游览器默认事件
  document.addEventListener("drop", forbidDefaultEvent);
  document.addEventListener("dragover", forbidDefaultEvent);
});
</script>

<template>
  <v-app>
    <v-app-bar color="" density="compact" data-tauri-drag-region>
      <v-app-bar-title>二次元图片放大✨</v-app-bar-title>

      <v-spacer></v-spacer>
      <Transition>
        <v-btn icon none-drag-region>
          <v-icon @click="changeTheme" v-if="isDark">mdi-weather-night</v-icon>
          <v-icon @click="changeTheme" v-else>mdi-weather-sunny</v-icon>
        </v-btn>
      </Transition>
      <v-btn icon none-drag-region>
        <v-icon @click="appWindow.minimize()">mdi-minus</v-icon>
      </v-btn>
      <v-btn icon none-drag-region>
        <v-icon @click="appWindow.toggleMaximize()">mdi-fullscreen</v-icon>
      </v-btn>

      <v-btn icon none-drag-region>
        <v-icon @click="appWindow.close()">mdi-close</v-icon>
      </v-btn>
    </v-app-bar>
    <!-- <v-main> -->
    <v-container class="" style="">
      <Suspense>
        <!-- 具有深层异步依赖的组件 -->
        <Real />

        <!-- 在 #fallback 插槽中显示 “正在加载中” -->
        <template #fallback> Loading... </template>
      </Suspense>
    </v-container>
    <!-- </v-main> -->
    <v-footer inset app> realcugan-ncnn-tauri@v0.2.0 </v-footer>
  </v-app>
</template>

<style>
.v-app-bar-title {
  user-select: none;
}

/* scrollbar */
html::-webkit-scrollbar {
  width: 10px;
  display: none;
}

html::-webkit-scrollbar-track {
  background: rgb(179, 177, 177);
  border-radius: 10px;
}

html::-webkit-scrollbar-thumb {
  background: rgb(136, 136, 136);
  border-radius: 10px;
}

html::-webkit-scrollbar-thumb:hover {
  background: rgb(100, 100, 100);
  border-radius: 10px;
}

html::-webkit-scrollbar-thumb:active {
  background: rgb(68, 68, 68);
  border-radius: 10px;
}
</style>
