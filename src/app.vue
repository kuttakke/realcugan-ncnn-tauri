<script setup>
import { open } from "@tauri-apps/api/shell";
import { appWindow } from "@tauri-apps/api/window";
import { useStorage } from "@vueuse/core";
import { onMounted, ref, watch } from "vue";
import { useTheme } from "vuetify";

import Real from "./modules/real/mod.vue";
import { isDark, toggleDark } from "./plugins/dark";

const theme = useTheme();
const myThemes = ["draculaDark", "nord"];
const magnification = useStorage("magnification", 2);
const thumbList = ["😄", "😁", "😍"];
const thumb = ref(thumbList[magnification.value - 2]);

const overlay = ref(false);

watch(magnification, (val) => {
  thumb.value = thumbList[val - 2];
});

const openGithub = async () => {
  await open("https://github.com/kuttakke/realcugan-ncnn-tauri");
};

const nowTheme = useStorage("theme", isDark.value ? "dark" : "light");
if (nowTheme.value.includes("dark") || nowTheme.value.includes("Dark")) {
  theme.global.name.value = myThemes[0];
} else {
  theme.global.name.value = myThemes[1];
}
// 阻止默认事件Function
const forbidDefaultEvent = (e) => {
  e.preventDefault();
};
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
  setDragAttr();
  // NOTE - 阻止游览器默认事件
  document.addEventListener("drop", forbidDefaultEvent);
  document.addEventListener("dragover", forbidDefaultEvent);
});
</script>

<template>
  <v-app>
    <v-app-bar color="" density="compact" data-tauri-drag-region>
      <v-app-bar-title>二次元风格图片放大{{ thumb }}</v-app-bar-title>

      <v-spacer></v-spacer>
      <Transition>
        <v-btn icon none-drag-region @click="changeTheme">
          <v-icon v-if="isDark">mdi-weather-night</v-icon>
          <v-icon v-else>mdi-weather-sunny</v-icon>
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
    <!-- <v-footer inset app> realcugan-ncnn-tauri@v0.2.2 </v-footer> -->
    <v-footer inset app>
      <div class="d-flex w-100 align-center">
        <strong>realcugan-ncnn-tauri</strong>

        <v-spacer></v-spacer>

        <v-btn
          class="mx-4"
          icon="mdi-github"
          variant="plain"
          size="small"
          @click="overlay = !overlay"
        ></v-btn>
      </div>
    </v-footer>
    <v-overlay
      :model-value="overlay"
      class="align-center justify-center"
      @click:outside="overlay = false"
    >
      <v-card v-intersect="onIntersect" class="mx-auto" max-width="336">
        <v-card-title :onclick="openGithub"
          >realcugan-ncnn-tauri@v0.2.4</v-card-title
        >
        <v-card-subtitle>降噪参数解释</v-card-subtitle>
        <v-card-text>
          <v-list lines="one">
            <v-list-item title="降噪版(1-3)：" subtitle=""
              ><p>
                如果原片噪声多，压得烂，推荐使用；目前2倍模型支持了3个降噪等级；
              </p></v-list-item
            >
            <v-list-item title="无降噪版(0)：" subtitle=""
              ><p>
                如果原片噪声不多，压得还行，但是想提高分辨率/清晰度/做通用性的增强、修复处理，推荐使用；
              </p></v-list-item
            >
            <v-list-item title="保守版(-1)：" subtitle=""
              ><p>
                如果你担心丢失纹理，担心画风被改变，担心颜色被增强，总之就是各种担心AI会留下浓重的处理痕迹，推荐使用该版本；但对于来源较模糊、渣清的，修复程度不会比降噪版更好。
              </p></v-list-item
            >
          </v-list>
        </v-card-text>
      </v-card>
    </v-overlay>
  </v-app>
</template>

<style>
.v-app-bar-title {
  user-select: none;
}
.v-footer {
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
