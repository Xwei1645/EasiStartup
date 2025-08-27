<template>
  <v-app>
    <v-main>
      <v-container fluid class="pa-6">
        <v-row>
          <v-col cols="12">
            <v-card>

              
              <v-card-text class="pa-6">
                <!-- 开机自启设置 -->
                <v-row class="mb-4">
                  <v-col cols="12">
                    <h3 class="text-h6 mb-4">启动设置</h3>
                    
                    <v-switch
                      v-model="settings.autoStart"
                      label="开机自启"
                      color="primary"
                      @change="updateAutoStart"
                      class="mb-3"
                    ></v-switch>
                    
                    <v-checkbox
                      v-model="settings.runAsAdmin"
                      label="以管理员身份启动"
                      color="primary"
                      :disabled="!settings.autoStart"
                      @change="updateRunAsAdmin"
                      class="mb-3 ml-6"
                    ></v-checkbox>
                    
                    <v-switch
                      v-model="settings.exitAfterStartup"
                      label="自启动项执行后退出"
                      color="primary"
                      @change="updateExitAfterStartup"
                      class="mb-3"
                    ></v-switch>
                    
                    <v-btn
                      variant="outlined"
                      color="warning"
                      size="small"
                      @click="restartAsAdmin"
                      class="mt-3"
                    >
                      <v-icon start>mdi-restart</v-icon>
                      以管理员权限重启
                    </v-btn>
                  </v-col>
                </v-row>
                
                <v-divider class="my-6"></v-divider>
                
                <!-- 关于部分 -->
                <v-row>
                  <v-col cols="12">
                    <h3 class="text-h6 mb-4">关于</h3>
                    
                    <v-card variant="outlined" class="pa-4">
                      <v-row align="center">
                        <v-col cols="auto">
                          <img src="/tauri.svg" alt="EasiStartup" width="64" height="64" />
                        </v-col>
                        <v-col>
                          <h4 class="text-h6 mb-1">EasiStartup</h4>
                          <p class="text-body-2 text-medium-emphasis mb-2">
                            版本 {{ appVersion }}
                          </p>
                          <p class="text-body-2 text-medium-emphasis mb-0">
                            一个简单易用的启动项管理工具
                          </p>
                        </v-col>
                      </v-row>
                      
                      <v-row class="mt-4">
                        <v-col cols="12">
                          <v-btn
                            variant="outlined"
                            color="primary"
                            size="small"
                            @click="openGitHub"
                          >
                            <v-icon start>mdi-github</v-icon>
                            GitHub
                          </v-btn>
                        </v-col>
                      </v-row>
                    </v-card>
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </v-main>
    <StartupReminder ref="startupReminderRef" />
  </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import StartupReminder from '../components/StartupReminder.vue';

// 设置数据
const settings = ref({
  autoStart: false,
  runAsAdmin: false,
  exitAfterStartup: false,
});

// 应用版本号
const appVersion = ref('0.1.0');

// 加载设置
const loadSettings = async () => {
  try {
    const savedSettings = await invoke('load_app_settings') as any;
    // 检查官方插件的自启动状态
    const autostartEnabled = await isEnabled();
    
    // 映射后端设置到前端设置
    settings.value = {
      autoStart: autostartEnabled || savedSettings.auto_startup_enabled,
      runAsAdmin: savedSettings.auto_startup_as_admin,
      exitAfterStartup: savedSettings.exit_after_startup
    };
  } catch (error) {
    console.error('加载设置失败:', error);
  }
};

// 获取应用版本
const getAppVersion = async () => {
  try {
    const version = await invoke('get_app_version') as string;
    appVersion.value = version;
  } catch (error) {
    console.error('获取版本号失败:', error);
  }
};

// 更新开机自启设置
const updateAutoStart = async () => {
  try {
    if (settings.value.autoStart) {
      // 启用自启动
      await enable();
    } else {
      // 禁用自启动
      await disable();
      // 如果禁用自启动，同时禁用管理员权限
      settings.value.runAsAdmin = false;
    }
    
    // 保存设置到本地配置
    const currentSettings = await invoke('load_app_settings') as any;
    const updatedSettings = {
      ...currentSettings,
      auto_startup_enabled: settings.value.autoStart,
      auto_startup_as_admin: settings.value.autoStart ? settings.value.runAsAdmin : false
    };
    await invoke('save_app_settings', { settings: updatedSettings });
    
    // 如果需要管理员权限，仍然使用原有的任务计划程序方式
    if (settings.value.autoStart && settings.value.runAsAdmin) {
      await invoke('apply_startup_settings', { settings: updatedSettings });
    }
  } catch (error) {
    console.error('更新开机自启设置失败:', error);
    // 如果失败，恢复原来的状态
    await loadSettings();
  }
};

// 更新管理员权限设置
const updateRunAsAdmin = async () => {
  // 如果开机自启未开启，不允许修改管理员权限
  if (!settings.value.autoStart) {
    return;
  }
  
  try {
    // 先加载当前设置
    const currentSettings = await invoke('load_app_settings') as any;
    // 更新管理员权限设置
    const updatedSettings = {
      ...currentSettings,
      auto_startup_as_admin: settings.value.runAsAdmin
    };
    // 保存设置
    await invoke('save_app_settings', { settings: updatedSettings });
    
    // 如果需要管理员权限，使用任务计划程序方式
    if (settings.value.runAsAdmin) {
      // 先禁用官方插件的自启动
      await disable();
      // 然后使用任务计划程序设置管理员权限自启动
      await invoke('apply_startup_settings', { settings: updatedSettings });
    } else {
      // 如果不需要管理员权限，使用官方插件
      await enable();
    }
  } catch (error) {
    console.error('更新管理员权限设置失败:', error);
    // 如果失败，恢复原来的状态
    await loadSettings();
  }
};

// 更新执行后退出设置
const updateExitAfterStartup = async () => {
  try {
    // 先加载当前设置
    const currentSettings = await invoke('load_app_settings') as any;
    // 更新执行后退出设置
    const updatedSettings = {
      ...currentSettings,
      exit_after_startup: settings.value.exitAfterStartup
    };
    // 保存设置
    await invoke('save_app_settings', { settings: updatedSettings });
  } catch (error) {
    console.error('更新执行后退出设置失败:', error);
    // 如果失败，恢复原来的状态
    await loadSettings();
  }
};

// 打开GitHub页面
const openGitHub = async () => {
  try {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl('https://github.com/Xwei1645/EasiStartup');
  } catch (error) {
    console.error('打开GitHub页面失败:', error);
  }
};

// 以管理员权限重启应用
const restartAsAdmin = async () => {
  try {
    await invoke('request_admin_restart');
  } catch (error) {
    console.error('以管理员权限重启失败:', error);
    // 可以在这里添加用户友好的错误提示
  }
};

const startupReminderRef = ref<InstanceType<typeof StartupReminder>>();

// 组件挂载时加载数据
onMounted(() => {
  loadSettings();
  getAppVersion();
  
  // 页面加载完成后检查自启动提醒
  setTimeout(() => {
    startupReminderRef.value?.checkReminders();
  }, 800);
});
</script>

<style scoped>
.v-switch {
  flex: none;
}
</style>