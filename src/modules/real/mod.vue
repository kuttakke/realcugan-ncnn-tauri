<!-- eslint-disable vue/multi-word-component-names -->

<script setup>
import "@uppy/core/dist/style.css";
import "@uppy/dashboard/dist/style.css";
import "@uppy/image-editor/dist/style.css";
import "@uppy/file-input/dist/style.css";
import "@uppy/status-bar/dist/style.min.css";
import "@uppy/screen-capture/dist/style.min.css";

import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import {
  BaseDirectory,
  createDir,
  exists,
  writeBinaryFile,
} from "@tauri-apps/api/fs";
import { resolveResource } from "@tauri-apps/api/path";
import { Command } from "@tauri-apps/api/shell";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { appWindow, PhysicalSize } from "@tauri-apps/api/window";
import Uppy from "@uppy/core";
import DashboardP from "@uppy/dashboard";
import ImageEditor from "@uppy/image-editor";
import Chinese from "@uppy/locales/lib/zh_CN";
import { Dashboard } from "@uppy/vue";
import { useStorage } from "@vueuse/core";
import { info } from "tauri-plugin-log-api";
import { error } from "tauri-plugin-log-api";
import { onMounted, onUnmounted, ref, watch } from "vue";
import { useToast } from "vue-toastification";

import { isDark } from "../../plugins/dark/index";

await appWindow.setMinSize(new PhysicalSize(960, 600));

const magnification = useStorage("magnification", 2);
const noise_level = useStorage("noise_level", -1);
const is_tta = useStorage("is_tta", false);
const format = useStorage("format", "png");
const is_run_all = useStorage("is_run_all", true);
const maxTasks = useStorage("max_tasks", 4);

const fakeNoiseLevelTick = {
  0: "-1",
  1: "0",
  2: "3",
};

const isShowFakeNoiseLevel = ref(false);
if (magnification.value !== 2) {
  isShowFakeNoiseLevel.value = true;
}

const chechPathExists = async () => {
  if (!(await exists("temp", { dir: BaseDirectory.Resource }))) {
    await createDir("temp", { dir: BaseDirectory.Resource });
  }
  if (!(await exists("result", { dir: BaseDirectory.Resource }))) {
    await createDir("result", { dir: BaseDirectory.Resource });
  }
};

watch(magnification, (newVal) => {
  if (newVal === 2) {
    isShowFakeNoiseLevel.value = false;
  } else {
    noise_level.value = parseInt(fakeNoiseLevelTick[fakeNoiseLevel.value]);
    isShowFakeNoiseLevel.value = true;
  }
});

const fakeNoiseLevel = ref(0);
watch(fakeNoiseLevel, (newVal) => {
  noise_level.value = parseInt(fakeNoiseLevelTick[newVal]);
});

const is_running = ref(false);
// const is_run_all = ref(true);
const uppy = new Uppy({
  restrictions: {
    allowedFileTypes: [".jpg", ".jpeg", ".png", ".webp"],
  },
  locale: Chinese,
  metaFields: [{ id: "name", name: "Name", placeholder: "file name" }],
});
uppy.use(DashboardP, {
  hideUploadButton: true,
});
uppy.use(ImageEditor, {
  quality: 0.8,
});

const resultFiles = ref([]);

const inserResultFiles = (img) => {
  // check is exist
  const index = resultFiles.value.findIndex((v) => v.mid === img.mid);
  if (index !== -1) {
    // console.log(index);
    resultFiles.value[index] = img;
  } else {
    resultFiles.value.push(img);
  }
};

const $message = useToast();

let unlisten = null;
let unlistenRes = null;

const changeFileProcessingStatus = (id, percent, status) => {
  const file = uppy.getFile(id);
  if (file != null) {
    uppy.setFileState(id, {
      progress: {
        percentage: percent,
        uploadStarted: status,
        bytesUploaded: (file.progress.bytesTotal * percent) / 100,
      },
    });
  }
};

const doneFileProcess = (id) => {
  const file = uppy.getFile(id);
  if (file != null) {
    uppy.setFileState(id, {
      progress: { uploadComplete: true, uploadStarted: true },
    });
  }
};

async function listen_run() {
  unlisten = await listen("run-status", async (event) => {
    changeFileProcessingStatus(event.payload.id, event.payload.percent, true);
  });
  unlistenRes = await listen("run-res", async (event) => {
    if (event.payload.status === false) {
      uppy.setFileState(event.payload.id, {
        progress: { uploadComplete: false, uploadStarted: false },
      });
    } else if (event.payload.status === true) {
      doneFileProcess(event.payload.id);
      try {
        const newUrl = convertFileSrc(
          rawResourcePath + "result\\" + event.payload.output
        );
        const newImage = new Image();
        newImage.src = newUrl;
        newImage.url = newUrl;
        newImage.resultFileName = event.payload.output;
        newImage.mid = event.payload.id;

        inserResultFiles(newImage);
      } catch (error) {
        $message.error("结果图片读取失败：" + event.payload.file);
      }
    }
  });
}

onMounted(() => {
  listen_run();
});

onUnmounted(() => {
  if (unlisten != null) {
    unlisten();
  }
  if (unlistenRes != null) {
    unlistenRes();
  }
});

const rawResourcePath = await resolveResource("");
const getresourceDir = () => {
  try {
    const appDataDirPath = rawResourcePath.replace("\\\\?\\", "");
    return appDataDirPath;
  } catch (e) {
    error(e);
    // console.log(e);
  }
};

const resourcePath = getresourceDir();
// const tempPath = resourcePath + "\\temp";
const resultPath = resourcePath + "\\result";

const openResult = async () => {
  const rp = resultPath.replace("\\\\", "\\").replace(/\\\\/g, "\\");
  await new Command("openResult", [rp]).execute();
};

// async function asyncPool(poolLimit, array, iteratorFn) {
//   const ret = []; //2
//   const executing = []; //3
//   for (const item of array) {
//     //4
//     const p = Promise.resolve().then(() => iteratorFn(item)); //5
//     ret.push(p); //6
//     if (poolLimit <= array.length) {
//       //7
//       const e = p.then(() => executing.splice(executing.indexOf(e), 1)); //8
//       executing.push(e); //9
//       if (executing.length >= poolLimit) {
//         //10
//         await Promise.race(executing); //11
//       }
//     }
//   }
//   return Promise.all(ret); //15
// }

const readFile = async (file) => {
  // return Array.from(new Uint8Array(await file.data.arrayBuffer()));
  const conetent = await file.data.arrayBuffer();
  // 保存至tempPath
  try {
    await writeBinaryFile(
      {
        path: "temp/" + file.meta.name,
        contents: new Uint8Array(conetent),
      },
      { dir: BaseDirectory.Resource }
    );
  } catch (e) {
    $message.error("图片保存失败: " + file.meta.name);
    return null;
  }
  return null;
};

const makeArgs = async () => {
  const ufiles = uppy.getFiles();
  const realFiles = ufiles.filter(
    (item) => !resultFiles.value.some((res) => res.mid === item.id)
  );
  if (realFiles.length === 0) {
    $message.error("没有需要处理的图片");
    return {
      argsList: [],
      maxTasks: maxTasks.value,
    };
  }
  const res = await Promise.all(
    [...realFiles].map(async (file) => {
      return {
        file_id: file.id,
        input_file: file.meta.name,
        input_file_data: await readFile(file),
        num_magnification: magnification.value,
        num_noise_level: noise_level.value,
        is_tta: is_tta.value,
        format_type: format.value,
      };
    })
  );
  // console.log(JSON.stringify(res));
  return {
    argsList: res,
    maxTasks: maxTasks.value,
    baseTime: new Date().getTime(),
  };
};
const devTime = ref(0);
const runTauriSideTask = async () => {
  devTime.value = 0;
  const interval = setInterval(() => {
    devTime.value += 100;
  }, 100);
  await chechPathExists();
  const args = await makeArgs();
  if (args.argsList.length === 0) {
    clearInterval(interval);
    return;
  }
  is_running.value = true;

  // then version
  invoke("run_all", args)
    .then((res) => {
      clearInterval(interval);
      is_running.value = false;
      $message.success("✨所有任务完成🎈\n耗时 " + devTime.value / 1000 + " s");
      info("webview task over:" + JSON.stringify(res));
      devTime.value = 0;
    })
    .catch((e) => {
      is_running.value = false;
      $message.error("运行错误" + e);
      clearInterval(interval);
      devTime.value = 0;
      // console.log(e);
    });
};
</script>

<template>
  <div>
    <div class="container-real">
      <v-navigation-drawer permanent floating>
        <v-list density="compact" nav color="transparent">
          <v-list-item title="" subtitle="">
            <v-slider
              color="primary"
              v-model="magnification"
              :ticks="[2, 3, 4]"
              step="1"
              max="4"
              min="2"
              show-ticks="always"
              label="放大倍数"
              style="height: 50px"
            >
            </v-slider>
          </v-list-item>
          <v-list-item title="">
            <v-slider
              v-if="!isShowFakeNoiseLevel"
              color="primary"
              v-model="noise_level"
              :ticks="[-1, 0, 1, 2, 3]"
              step="1"
              max="3"
              min="-1"
              show-ticks="always"
              label="降噪等级"
              style="height: 50px"
            ></v-slider>
            <v-slider
              v-else
              color="primary"
              v-model="fakeNoiseLevel"
              :ticks="fakeNoiseLevelTick"
              step="1"
              max="2"
              min="0"
              show-ticks="always"
              label="降噪等级"
              style="height: 50px"
            ></v-slider>
          </v-list-item>
          <v-divider></v-divider>
          <v-list-item>
            <v-switch
              v-model="is_tta"
              label="TTA模式"
              inset
              color="primary"
            ></v-switch>
            <template v-slot:append>
              <v-tooltip activator="parent" location="right">
                <!-- eslint-disable-next-line vue/no-unused-vars -->
                <template v-slot:activator="{ on }">
                  <v-icon color="primary" class="text-center">
                    mdi-information
                  </v-icon>
                </template>
                <span
                  >转换时间较未选中时增加八倍，提高0.15的峰值信噪比（PSNR），效果可能不明显</span
                >
              </v-tooltip>
            </template>
          </v-list-item>
          <v-list-item>
            <v-switch
              v-model="is_run_all"
              label="快速模式"
              inset
              color="primary"
            ></v-switch>
            <template v-slot:append>
              <v-tooltip activator="parent" location="right">
                <!-- eslint-disable-next-line vue/no-unused-vars -->
                <template v-slot:activator="{ on }">
                  <v-icon color="primary"> mdi-information </v-icon>
                </template>
                并行执行(最大4任务)，报错请关闭</v-tooltip
              >
            </template>
          </v-list-item>
          <v-list-item title="" subtitle="">
            <v-select
              v-model="format"
              :items="['png', 'jpg', 'webp']"
              label="输出格式"
            ></v-select>
          </v-list-item>
          <v-list-item title="" class="mt-3">
            <v-btn
              color="primary"
              @click="runTauriSideTask"

              :disabled="is_running"
              :outlined="true"
            >
              <span class="hidden-sm-and-down">{{ devTime === 0 ? "开始" : (devTime / 1000) + "s"  }}</span>
            </v-btn>
            <template v-slot:append>
              <v-btn @click="openResult" color="primary">
                <span class="hidden-sm-and-down">打开结果文件夹</span>
              </v-btn>
            </template>
          </v-list-item>
        </v-list>

        <v-divider></v-divider>
      </v-navigation-drawer>
    </div>
    <v-main class="content">
      <div class="dashboard-content">
        <dashboard
          ref="dash"
          :uppy="uppy"
          :props="{
            metaFields: [
              { id: 'name', name: 'Name', placeholder: 'file name' },
            ],
            width: 600,
            height: 300,
            hideUploadButton: true,
            disableStatusBar: true,
            locale: {
              strings: {
                uploadFailed: '放大失败',
                filesUploadedOfTotal:
                  '已放大 %{smart_count} 个图片中的 %{complete} 个',
                xMoreFilesAdded: '又有 %{smart_count} 个图片被添加',
                upload: '开始放大',
                uploading: '放大中',
                uploadingXFiles: '正在放大 %{smart_count} 个图片',
                uploadXNewFiles: '新放大了 %{smart_count} 个文件',
                xFilesSelected: '%{smart_count} 个图片等待放大',
                poweredBy: '',
              },
            },
            theme: isDark ? 'dark' : 'light',
          }"
          :plugins="['ImageEditor']"
        >
        </dashboard>
      </div>
      <v-divider class="mt-4"></v-divider>
      <!-- 图片结果展示 -->
      <div class="magnification-images" v-show="resultFiles.length">
        <!-- <v-row v-viewer transition-duration="0.3s"> -->
        <masonry
          v-viewer
          transition-duration="0.3s"
          :cols="{ default: 3, 700: 2, 400: 1 }"
          :gutter="30"
        >
          <v-card
            class="pb-1 mt-2"
            v-for="(item, index) in resultFiles"
            :key="item.mid"
          >
            <v-img :src="item.src">
              <template v-slot:placeholder>
                <v-row class="fill-height ma-0" align="center" justify="center">
                  <v-progress-circular
                    indeterminate
                    color="grey-lighten-5"
                  ></v-progress-circular>
                </v-row>
              </template>
            </v-img>
            <v-card-text>{{ item.resultFileName }}</v-card-text>
          </v-card>
        </masonry>
      </div>
      <br />
    </v-main>
  </div>
</template>

<style>
.v-input__details {
  display: none !important;
}

.v-list-item__content {
  overflow: visible !important;
}

.container-real {
  user-select: none !important;
}

.v-card .v-img {
  user-select: none !important;
}

.uppy-Dashboard-Item-preview {
  user-select: none !important;
}

.dashboard-content {
  justify-items: center;
}

.v-container {
  max-width: max-content !important;
}

.uppy-Dashboard-AddFiles-title {
  user-select: none;
}
</style>
