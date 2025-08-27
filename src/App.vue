<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { getCurrentWindow } from '@tauri-apps/api/window';

const router = useRouter();

// 根据窗口标识符导航到对应页面
onMounted(async () => {
  try {
    const currentWindow = getCurrentWindow();
    const windowLabel = currentWindow.label;
    router.push('/' + windowLabel);
  } catch (error) {
    console.error('获取窗口信息失败:', error);
    // 默认导航到启动项编辑页面
    router.push('/startup-editor');
  }
});
</script>

<template>
  <router-view />
</template>

<style>
/* 全局样式由Vuetify提供 */
</style>