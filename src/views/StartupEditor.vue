<template>
  <v-app>
    <v-main>
      <v-container fluid class="pa-6">
        <v-row>
          <v-col cols="12">
            <!-- 页面标题和按钮组 -->
            <div class="d-flex justify-space-between align-center mb-6">
              <h2 class="text-h4">启动项编辑</h2>
              <div class="d-flex" style="gap: 12px;">
                 <v-btn
                   color="primary"
                   variant="elevated"
                   @click="executeAllItems"
                   prepend-icon="mdi-play-circle"
                   :disabled="startupItems.length === 0 || isExecuting"
                   :loading="isExecuting"
                 >
                   全部运行
                 </v-btn>
                 <v-btn
                   color="primary"
                   variant="elevated"
                   @click="addStartupItem"
                   prepend-icon="mdi-plus"
                 >
                   添加启动项
                 </v-btn>
               </div>
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
                      <v-card v-if="item.executablePath" variant="outlined" class="mb-3">
                        <v-card-text class="pa-3">
                          <div class="d-flex">
                            <div class="d-flex flex-column justify-space-between mr-3" style="height: 48px;">
                              <v-avatar size="32">
                                <v-img
                                  v-if="item.icon"
                                  :src="item.icon"
                                  alt="应用图标"
                                  @error="handleIconError(index)"
                                ></v-img>
                                <v-icon v-else size="small">mdi-application</v-icon>
                              </v-avatar>
                            </div>
                            <div class="flex-grow-1 d-flex flex-column justify-space-between" style="height: 48px;">
                              <div class="text-subtitle-2 font-weight-medium">{{ item.displayName }}</div>
                              <div class="text-caption text-medium-emphasis" :title="item.executablePath">{{ truncatePath(item.executablePath || '') }}</div>
                            </div>
                          </div>
                        </v-card-text>
                      </v-card>
                      
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
                  
                  <!-- 卡片底部操作按钮 -->
                  <v-card-actions class="pa-3 pt-0">
                    <v-spacer></v-spacer>
                    <v-btn
                       color="primary"
                       variant="outlined"
                       size="small"
                       @click="executeItem(index)"
                       prepend-icon="mdi-play"
                       :disabled="!item.enabled || (item.mode === 'normal' && !item.executablePath) || (item.mode === 'command' && !item.command) || executingItems.has(index)"
                       :loading="executingItems.has(index)"
                     >
                       运行
                     </v-btn>
                  </v-card-actions>
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

// 执行状态
const isExecuting = ref(false);
const executingItems = ref(new Set<number>());

// 省略过长路径的函数
const truncatePath = (path: string, maxLength: number = 50): string => {
  if (!path || path.length <= maxLength) return path;
  
  const parts = path.split('\\');
  if (parts.length <= 2) return path;
  
  let result = parts[parts.length - 1]; // 文件名
  let currentLength = result.length;
  
  // 从右往左添加目录
  for (let i = parts.length - 2; i >= 0; i--) {
    const part = parts[i];
    const newLength = currentLength + part.length + 1; // +1 for \\
    
    if (newLength > maxLength && i > 0) {
      result = '...' + '\\' + result;
      break;
    }
    
    result = part + '\\' + result;
    currentLength = newLength;
  }
  
  return result;
};

// 添加新的启动项
const addStartupItem = async () => {
  try {
    const newItem = await invoke('create_startup_item') as StartupItem;
    newItem.name = '';
    startupItems.value.push(newItem);
    await saveAllItems();
  } catch (error) {
    console.error('创建启动项失败:', error);
  }
};

// 选择可执行文件
const selectExecutable = async (index: number) => {
  try {
    const selectedPath = await invoke('open_file_dialog') as string | null;
    if (selectedPath) {
      const item = startupItems.value[index];
      let actualPath = selectedPath;
      let displayName = '';
      
      // 检查是否是快捷方式
      if (selectedPath.toLowerCase().endsWith('.lnk')) {
        try {
          const shortcutInfo = await invoke('get_shortcut_info', { executablePath: selectedPath }) as [string, string] | null;
          if (shortcutInfo) {
            const [targetPath, shortcutDisplayName] = shortcutInfo;
            actualPath = targetPath;
            displayName = shortcutDisplayName;
            
            // 如果名称为空，自动填入快捷方式的标题
            if (!item.name) {
              item.name = shortcutDisplayName;
            }
          }
        } catch (shortcutError) {
          console.warn('获取快捷方式信息失败:', shortcutError);
        }
      }
      
      // 如果不是快捷方式或快捷方式解析失败，从路径中提取文件名
      if (!displayName) {
        const fileName = actualPath.split('\\').pop() || actualPath.split('/').pop() || '';
        displayName = fileName.replace(/\.[^/.]+$/, '');
        
        // 如果名称为空，自动填入文件名
        if (!item.name) {
          item.name = displayName;
        }
      }
      
      item.executablePath = actualPath;
      item.displayName = displayName;
      
      // 获取可执行文件图标（使用实际路径）
      try {
        const iconPath = await invoke('get_executable_icon', { executablePath: actualPath }) as string | null;
        if (iconPath) {
          item.icon = iconPath;
        } else {
          // 图标获取失败，清空icon字段以使用默认图标
          item.icon = '';
        }
      } catch (iconError) {
        console.warn('获取图标失败:', iconError);
        // 图标获取失败，清空icon字段以使用默认图标
        item.icon = '';
      }
      
      await saveAllItems();
      console.log('选择可执行文件:', item);
    }
  } catch (error) {
    console.error('选择文件失败:', error);
  }
};



// 删除启动项
const deleteItem = async (index: number) => {
  startupItems.value.splice(index, 1);
  await saveAllItems();
};

// 处理图标加载错误
const handleIconError = (index: number) => {
  const item = startupItems.value[index];
  if (item) {
    console.warn('图标加载失败，使用默认图标:', item.icon);
    item.icon = ''; // 清空图标路径，使用默认图标
  }
};

// 保存单个启动项
const saveItem = async (index: number) => {
  await saveAllItems();
};

// 保存所有启动项
const saveAllItems = async () => {
  try {
    await invoke('save_startup_items', { items: startupItems.value });
    console.log('保存启动项列表成功');
  } catch (error) {
    console.error('保存启动项失败:', error);
  }
};

// 加载启动项列表
const loadStartupItems = async () => {
  try {
    const items = await invoke('load_startup_items') as StartupItem[];
    startupItems.value = items;
    console.log('加载启动项列表成功:', items);
  } catch (error) {
    console.error('加载启动项列表失败:', error);
    startupItems.value = [];
  }
};

// 执行单个启动项
const executeItem = async (index: number) => {
  const item = startupItems.value[index];
  if (!item.enabled) return;
  
  executingItems.value.add(index);
  
  try {
    await invoke('execute_startup_item', { item });
    console.log('执行启动项成功:', item.name);
  } catch (error) {
    console.error('执行启动项失败:', error);
  } finally {
    executingItems.value.delete(index);
  }
};

// 执行所有启动项
const executeAllItems = async () => {
  isExecuting.value = true;
  
  try {
    await invoke('execute_all_startup_items');
    console.log('执行所有启动项成功');
  } catch (error) {
    console.error('执行所有启动项失败:', error);
  } finally {
    isExecuting.value = false;
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