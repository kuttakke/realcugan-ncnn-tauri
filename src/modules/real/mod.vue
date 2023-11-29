<script setup>
import { ref, computed } from "vue";
import { LazyImg, Waterfall } from "vue-waterfall-plugin-next";
import {
  writeBinaryFile,
  readBinaryFile,
  BaseDirectory,
  createDir,
  exists,
} from "@tauri-apps/api/fs";
import { useToast } from "vue-toastification";
import "vue-waterfall-plugin-next/dist/style.css";
const magnification = ref(2);
const noise_level = ref(-1);
const is_tta = ref(false);
const format = ref("png");
const files = ref([]);
const is_running = ref(false);
const is_run_all = ref(true);

const resultFiles = ref([]);

const $message = useToast();

const progressLinearVal = computed(() => {
  return (100 * resultFiles.value.length) / files.value.length;
});

const getresourceDir = async () => {
  try {
    const appDataDirPath = (await window.__TAURI__.path.resourceDir()).replace(
      "\\\\?\\",
      ""
    );

    console.log(appDataDirPath);
    console.log(typeof appDataDirPath);
    return appDataDirPath;
  } catch (e) {
    console.log(e);
  }
};

const chechPathExists = async () => {
  if (!(await exists("temp", { dir: BaseDirectory.Resource }))) {
    console.log("create temp path");
    await createDir("temp", { dir: BaseDirectory.Resource });
  }
  if (!(await exists("result", { dir: BaseDirectory.Resource }))) {
    console.log("create result path");
    await createDir("result", { dir: BaseDirectory.Resource });
  }
};

const resourcePath = await getresourceDir();
const tempPath = resourcePath + "\\temp";
const resultPath = resourcePath + "\\result";

const checkIsImage = (files) => {
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    if (!file.type.match("image.*")) {
      console.log("not image file here");
      return false;
    }
    file.url = window.URL.createObjectURL(file);
    file.src = file.url;
    console.log(file.url);
  }
  return true;
};

const rules = [(file) => checkIsImage(file) || "请选择图片文件"];

const removeFile = (file) => {
  console.log("remove file");
  console.log(file);
  files.value.splice(files.value.indexOf(file), 1);
};

const getNewFilePath = (file) => {
  let lastDotIndex = file.name.lastIndexOf(".");
  let fileNameWithoutExtension = "";
  if (lastDotIndex !== -1) {
    // 移除最后一个 . 以及其后的部分
    fileNameWithoutExtension = file.name.slice(0, lastDotIndex);
  } else {
    fileNameWithoutExtension = file.name;
  }
  file.resultFileName =
    magnification.value +
    "_" +
    noise_level.value +
    "_" +
    fileNameWithoutExtension +
    "." +
    format.value;

  file.resultFilePath = resultPath + "\\" + file.resultFileName;
  console.log("result name:", file.resultFilePath);
};

const openResult = async () => {
  const rp = resultPath.replace("\\\\", "\\").replace(/\\\\/g, "\\");
  console.log(rp);
  console.log([rp]);
  await new window.__TAURI__.shell.Command("openResult", [rp]).execute();
};

const invokeMagnification = async (file) => {
  getNewFilePath(file);
  const commandStr = [
    "-i",
    tempPath + "\\" + file.name,
    "-o",
    file.resultFilePath,
    "-n",
    "" + noise_level.value,
    "-f",
    format.value,
    "-s",
    "" + magnification.value,
    is_tta.value ? "-x" : "",
  ];
  console.log(commandStr);
  try {
    const output = await new window.__TAURI__.shell.Command(
      "realcugan",
      commandStr
    ).execute();
    console.log(output.stderr);
    console.log(output.stdout);
    console.log(output.code);
    $message.success("图片处理成功: " + file.name);
    return true;
  } catch (e) {
    console.log(e);
    $message.error("图片处理失败: " + file.name);
    return false;
  }
};

// console.log(currentDir);
const magnificationOne = async (file) => {
  const conetent = await file.arrayBuffer();
  // 保存至tempPath
  try {
    await writeBinaryFile(
      {
        path: "temp/" + file.name,
        contents: new Uint8Array(conetent),
      },
      { dir: BaseDirectory.Resource }
    );
    console.log("save file at", tempPath + "\\" + file.name);
    $message.info("图片保存成功: " + file.name);
  } catch (e) {
    console.log("save file error", e);
    return;
  }
  await invokeMagnification(file);

  try {
    // return a Promise<Uint8Array>
    const newFileBinary = await readBinaryFile(
      "result/" + file.resultFileName,
      { dir: BaseDirectory.Resource }
    );
    // create a new url for the file
    const newFileUrl = URL.createObjectURL(new Blob([newFileBinary]));
    // create a new file object and add to resultFiles
    const newFile = new File([newFileBinary], file.resultFileName, {
      type: file.type,
    });
    newFile.url = newFileUrl;
    newFile.src = newFile.url;
    resultFiles.value.push(newFile);
    $message.success("结果图片保存成功: " + file.name);
  } catch (e) {
    console.log("read file error", e);
    $message.error("结果图片保存失败: " + file.name);
  }
};

const startMagnification = async () => {
  console.log("start magnification");
  is_running.value = true;
  await chechPathExists();
  if (is_run_all.value) {
    await Promise.all(
      files.value.map(async (file) => {
        await magnificationOne(file);
      })
    );
  } else {
    for (const file of files.value) {
      await magnificationOne(file);
    }
  }

  console.log("end magnification");
  is_running.value = false;
};
</script>

<template>
  <div class="container">
    <v-navigation-drawer class="bg-deep-purple" theme="dark" permanent>
      <v-card title="二次元图片放大✨"></v-card>
      <v-list density="compact" nav color="transparent" lines="two">
        <v-list-item title="" subtitle="">
          <v-slider
            v-model="magnification"
            :ticks="[2, 3, 4]"
            step="1"
            max="4"
            min="2"
            show-ticks="always"
            label="放大倍数"
          ></v-slider>
        </v-list-item>
        <v-list-item>
          <v-slider
            v-model="noise_level"
            :ticks="[-1, 0, 1, 2, 3]"
            step="1"
            max="3"
            min="-1"
            show-ticks="always"
            label="降噪等级"
          ></v-slider>
        </v-list-item>
        <v-divider></v-divider>
        <v-list-item>
          <v-switch v-model="is_tta" label="TTA模式" inset></v-switch>
          <template v-slot:append>
            <v-btn icon="mdi-information">
              <v-tooltip activator="parent" location="top">
                <template v-slot:activator="{ on }">
                  <v-icon v-on="on" color="primary" dark>
                    mdi-information
                  </v-icon> </template
                >不是很清楚是干什么用的</v-tooltip
              >
            </v-btn>
          </template>
        </v-list-item>
        <v-list-item>
          <v-switch v-model="is_run_all" label="快速模式" inset></v-switch>
          <template v-slot:append>
            <v-btn icon="mdi-information">
              <v-tooltip activator="parent" location="top">
                <template v-slot:activator="{ on }">
                  <v-icon v-on="on" color="primary" dark>
                    mdi-information
                  </v-icon>
                </template>
                显存要是爆了就关了吧</v-tooltip
              >
            </v-btn>
          </template>
        </v-list-item>
        <v-list-item title="" subtitle="">
          <v-select
            v-model="format"
            :items="['png', 'jpg', 'webp']"
            label="输出格式"
          ></v-select>
        </v-list-item>
        <v-list-item title="">
          <v-btn
            @click="startMagnification"
            :loading="is_running"
            :disabled="is_running"
            color="deep-purple-accent-4"
            :outlined="true"
          >
            <span class="hidden-sm-and-down">开始</span>
          </v-btn>
          <template v-slot:append>
            <v-btn color="deep-purple-accent-4" @click="openResult">
              <span class="hidden-sm-and-down">打开结果文件夹</span>
            </v-btn>
          </template>
        </v-list-item>
      </v-list>

      <v-divider></v-divider>
    </v-navigation-drawer>
  </div>
  <div class="content">
    <div class="file-drop-box">
      <p class="text-h5">上传图片文件，可拖拽</p>
    </div>
    <br />
    <v-file-input
      v-model="files"
      color="deep-purple-accent-4"
      :rules="rules"
      counter
      label="File input"
      multiple
      placeholder="Select your files"
      accept="image/png, image/jpeg, image/jpg"
      prepend-icon="mdi-image"
      variant="outlined"
      :active="!is_running"
      :show-size="1000"
    >
      <template v-slot:selection="{ fileNames }">
        <template v-for="(fileName, index) in fileNames" :key="fileName">
          <v-chip
            v-if="index < 2"
            color="deep-purple-accent-4"
            label
            size="small"
            class="me-2"
          >
            {{ fileName }}
          </v-chip>

          <span
            v-else-if="index === 2"
            class="text-overline text-grey-darken-3 mx-2"
          >
            +{{ files.length - 2 }} File(s)
          </span>
        </template>
      </template>
    </v-file-input>
    <br />
    <v-progress-linear
      v-model="progressLinearVal"
      color="blue-grey"
      height="25"
      v-if="is_running"
      striped
    >
      <template v-slot:default="{ value }">
        <strong>{{ Math.ceil(value) }}%</strong>
      </template>
    </v-progress-linear>
    <!-- 图片卡片展示 -->
    <v-sheet
      class="mx-auto"
      elevation="8"
      max-width="800"
      v-show="files.length"
    >
      <v-slide-group class="pa-4" center-active show-arrows>
        <v-slide-group-item
          v-for="(file, n) in files"
          :key="n"
          v-slot="{ isSelected, toggle }"
        >
          <v-card
            :color="isSelected ? 'primary' : 'grey-lighten-1'"
            class="ma-4 slide-group-item-t"
            height="200"
            width="100"
            :image="file.url"
            @click="toggle"
          >
            <div class="d-flex fill-height align-center justify-center">
              <v-scale-transition>
                <v-icon
                  v-if="isSelected"
                  color="red"
                  size="48"
                  icon="mdi-close-circle-outline"
                  @click="removeFile(file)"
                ></v-icon>
              </v-scale-transition>
            </div>
          </v-card>
        </v-slide-group-item>
      </v-slide-group>
    </v-sheet>
    <div class="magnification-images" v-show="resultFiles.length">
      <Waterfall :list="resultFiles">
        <template #item="{ item, url, index }">
          <v-expand-transition>
            <v-card>
              <LazyImg :url="url" />
              <p class="text">{{ item.name }}</p>
            </v-card>
          </v-expand-transition>
        </template>
      </Waterfall>
    </div>
  </div>
</template>

<style scoped></style>
