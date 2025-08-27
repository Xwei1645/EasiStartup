<template>
  <v-app>
    <v-main>
      <v-container fluid class="pa-6">
        <v-row>
          <v-col cols="12">
            <!-- 页面标题和添加按钮 -->
            <div class="d-flex justify-space-between align-center mb-6">
              <h2 class="text-h4">启动项编辑</h2>
              <v-btn
                color="primary"
                variant="elevated"
                @click="addStartupItem"
                prepend-icon="mdi-plus"
              >
                添加启动项
              </v-btn>
            </div>
            
            <!-- 启动项列表 -->
            <v-row v-if="startupItems.length > 0">
              <v-col
                v-for="(item, index) in startupItems"
                :key="item.id"
                cols="12"
                md="6"
                lg="4"
              >
                <v-card class="startup-item-card" elevation="2">
                  <v-card-title class="d-flex justify-space-between align-center pa-3">
                    <div class="d-flex align-center flex-grow-1">
                      <v-switch
                        v-model="item.enabled"
                        color="primary"
                        density="compact"
                        hide-details
                        class="mr-3"
                        @update:model-value="saveItem(index)"
                      ></v-switch>
                      <v-text-field
                        v-model="item.name"
                        variant="plain"
                        density="compact"
                        hide-details
                        placeholder="未命名启动项"
                        class="title-input flex-grow-1"
                        @blur="saveItem(index)"
                      ></v-text-field>
                      <v-btn
                        v-if="item.mode === 'normal'"
                        icon="mdi-folder-open"
                        variant="text"
                        size="small"
                        color="primary"
                        @click="selectExecutable(index)"
                        class="ml-2"
                      ></v-btn>
                    </div>
                    <v-btn
                      icon="mdi-delete"
                      variant="text"
                      size="small"
                      color="error"
                      @click="deleteItem(index)"
                    ></v-btn>
                  </v-card-title>
                  
                  <v-card-text class="pa-3">
                    <!-- 模式切换 -->
                    <v-chip-group
                      v-model="item.mode"
                      mandatory
                      class="mb-3"
                      @update:model-value="saveItem(index)"
                    >
                      <v-chip value="normal" variant="outlined" size="small">
                        <v-icon start size="small">mdi-application</v-icon>
                        常规模式
                      </v-chip>
                      <v-chip value="command" variant="outlined" size="small">
                        <v-icon start size="small">mdi-console</v-icon>
                        命令模式
                      </v-chip>
                    </v-chip-group>
                    
                    <!-- 常规模式 -->
                    <div v-if="item.mode === 'normal'">
                      <!-- 显示文件信息 -->
                      <div v-if="item.executablePath" class="d-flex align-center mb-3">
                        <v-avatar size="24" class="mr-2">
                          <v-img
                            v-if="item.icon"
                            :src="item.icon"
                            alt="应用图标"
                          ></v-img>
                          <v-icon v-else size="small">mdi-application</v-icon>
                        </v-avatar>
                        <div class="text-body-2">{{ item.displayName }}</div>
                      </div>
                      
                      <!-- 启动参数 -->
                      <v-text-field
                        v-model="item.arguments"
                        label="启动参数（可选）"
                        placeholder="例如: --minimized"
                        density="compact"
                        variant="outlined"
                        class="mb-3"
                        @blur="saveItem(index)"
                      ></v-text-field>
                    </div>
                    
                    <!-- 命令模式 -->
                    <div v-if="item.mode === 'command'">
                      <v-textarea
                        v-model="item.command"
                        label="命令"
                        placeholder="输入要执行的命令"
                        :rows="Math.min(Math.max(2, Math.ceil((item.command || '').length / 50)), 8)"
                        auto-grow
                        :max-rows="8"
                        density="compact"
                        variant="outlined"
                        class="mb-3"
                        @blur="saveItem(index)"
                      ></v-textarea>
                    </div>
                    
                    <!-- 启动选项（可折叠） -->
                    <v-expansion-panels variant="accordion" class="mt-3">
                      <v-expansion-panel>
                        <v-expansion-panel-title class="text-body-2 pa-2">
                          <v-icon start size="small">mdi-cog</v-icon>
                          启动选项
                        </v-expansion-panel-title>
                        <v-expansion-panel-text class="pa-2">
                          <v-checkbox
                            v-model="item.runAsAdmin"
                            label="以管理员身份运行"
                            color="primary"
                            density="compact"
                            hide-details
                            class="mb-2"
                            @update:model-value="saveItem(index)"
                          ></v-checkbox>
                          
                          <!-- 延迟启动 -->
                           <div class="d-flex align-center mb-2">
                             <v-checkbox
                               v-model="item.delayEnabled"
                               label="延迟启动"
                               color="primary"
                               density="compact"
                               hide-details
                               class="mr-3"
                               @update:model-value="saveItem(index)"
                             ></v-checkbox>
                             
                             <v-text-field
                               v-if="item.delayEnabled"
                               v-model.number="item.delaySeconds"
                               type="number"
                               style="width: 80px"
                               density="compact"
                               variant="outlined"
                               suffix="秒"
                               :min="1"
                               :max="300"
                               hide-details
                               @blur="saveItem(index)"
                             ></v-text-field>
                           </div>
                        </v-expansion-panel-text>
                      </v-expansion-panel>
                    </v-expansion-panels>
                  </v-card-text>
                  

                </v-card>
              </v-col>
            </v-row>
            
            <!-- 空状态 -->
            <div v-else class="text-center pa-8">
              <v-icon size="64" color="grey-lighten-1" class="mb-4">mdi-rocket-launch-outline</v-icon>
              <h3 class="text-h6 mb-2">还没有启动项</h3>
              <p class="text-body-2 text-medium-emphasis">
                点击右上角的"添加启动项"按钮来创建您的第一个启动项
              </p>
            </div>
          </v-col>
        </v-row>
      </v-container>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 启动项数据类型
interface StartupItem {
  id: string;
  name: string;
  mode: 'normal' | 'command';
  executablePath?: string;
  displayName?: string;
  icon?: string;
  arguments?: string;
  command?: string;
  runAsAdmin: boolean;
  enabled: boolean;
  delayEnabled: boolean;
  delaySeconds: number;
}

// 启动项列表
const startupItems = ref<StartupItem[]>([]);

// 生成唯一ID
const generateId = (): string => {
  return Date.now().toString(36) + Math.random().toString(36).substr(2);
};

// 添加新的启动项
const addStartupItem = () => {
  const newItem: StartupItem = {
    id: generateId(),
    name: '',
    mode: 'normal',
    executablePath: '',
    displayName: '',
    icon: '',
    arguments: '',
    command: '',
    runAsAdmin: false,
    enabled: true,
    delayEnabled: false,
    delaySeconds: 5,
  };
  
  startupItems.value.push(newItem);
};

// 选择可执行文件
const selectExecutable = async (index: number) => {
  try {
    // 这里将来会调用Tauri的文件选择对话框
    // const selected = await invoke('select_executable_file');
    // if (selected) {
    //   const item = startupItems.value[index];
    //   item.executablePath = selected.path;
    //   item.displayName = selected.name;
    //   item.icon = selected.icon;
    //   item.name = selected.name;
    // }
    
    // 临时模拟数据
    const item = startupItems.value[index];
    item.executablePath = 'C:\\Program Files\\Notepad++\\notepad++.exe';
    item.displayName = 'Notepad++';
    item.name = 'Notepad++';
    console.log('选择可执行文件:', item);
  } catch (error) {
    console.error('选择文件失败:', error);
  }
};



// 删除启动项
const deleteItem = (index: number) => {
  startupItems.value.splice(index, 1);
};

// 保存启动项
const saveItem = async (index: number) => {
  try {
    const item = startupItems.value[index];
    // 这里将来会调用后端API保存启动项
    // await invoke('save_startup_item', { item });
    console.log('保存启动项:', item);
  } catch (error) {
    console.error('保存启动项失败:', error);
  }
};

// 加载启动项列表
const loadStartupItems = async () => {
  try {
    // 这里将来会调用后端API获取启动项列表
    // const items = await invoke('get_startup_items');
    // startupItems.value = items;
    
    // 临时模拟数据
    startupItems.value = [];
    
    console.log('加载启动项列表');
  } catch (error) {
    console.error('加载启动项列表失败:', error);
  }
};

// 组件挂载时加载数据
onMounted(() => {
  loadStartupItems();
});
</script>

<style scoped>
.startup-item-card {
  height: 100%;
  transition: transform 0.2s ease-in-out;
}

.startup-item-card:hover {
  transform: translateY(-2px);
}

.v-chip-group {
  justify-content: center;
}

.title-input :deep(.v-field__input) {
  font-weight: 500;
  font-size: 1.1rem;
}

.title-input :deep(.v-field__field) {
  padding: 0;
}

.v-expansion-panels {
  box-shadow: none;
}

.v-expansion-panel {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  border-radius: 4px;
}
</style>