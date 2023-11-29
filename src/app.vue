<script setup>
/* template specific */

// import Counter from './modules/counter/mod.vue';
// import Fetcher from './modules/fetcher/mod.vue';
import Real from "./modules/real/mod.vue";

// import { defineAsyncComponent } from 'vue'

// const Real = defineAsyncComponent(() => import('./modules/real/mod.vue'))
</script>

<template>
  <v-app>
    <v-main class="bg-grey-lighten-4">
      <v-container class="mx-auto" style="padding: 50px">
        <!-- <counter />

        <fetcher class="mt-3" /> -->
        <!-- <Real></Real> -->
        <Suspense>
          <!-- 具有深层异步依赖的组件 -->
          <Real />

          <!-- 在 #fallback 插槽中显示 “正在加载中” -->
          <template #fallback> Loading... </template>
        </Suspense>
      </v-container>
    </v-main>
  </v-app>
</template>
