<template>
  <Transition name="fade" appear>
    <div v-if="showDialog" class="reminder-overlay">
      <Transition name="slide-up" appear>
        <div v-if="showDialog" class="reminder-dialog">
          <div class="reminder-header">
            <h3>{{ dialogTitle }}</h3>
            <button @click="closeDialog" class="close-btn">&times;</button>
          </div>
          
          <div class="reminder-content">
            <div class="reminder-icon">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M12 2L13.09 8.26L22 9L13.09 9.74L12 16L10.91 9.74L2 9L10.91 8.26L12 2Z" fill="#fbbf24"/>
                <path d="M12 7C14.76 7 17 9.24 17 12S14.76 17 12 17 7 14.76 7 12 9.24 7 12 7Z" fill="#f59e0b"/>
                <path d="M12 9C13.66 9 15 10.34 15 12S13.66 15 12 15 9 13.66 9 12 10.34 9 12 9Z" fill="#d97706"/>
              </svg>
            </div>
            
            <p class="reminder-message">{{ dialogMessage }}</p>
            
            <div class="reminder-actions">
              <button @click="enableStartup" class="btn btn-primary">
                {{ actionButtonText }}
              </button>
              <button @click="closeDialog" class="btn btn-secondary">
                稍后设置
              </button>
            </div>
            
            <div class="reminder-checkbox">
              <label>
                <input 
                  type="checkbox" 
                  v-model="dontShowAgain"
                />
                不再显示此提醒
              </label>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Props {
  show?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  show: false
})

const emit = defineEmits<{
  close: []
  startupEnabled: []
}>()

const showDialog = ref(false)
const reminderType = ref<'startup' | 'admin' | null>(null)
const dontShowAgain = ref(false)
const windowReady = ref(false)

const dialogTitle = computed(() => {
  switch (reminderType.value) {
    case 'startup':
      return '自启动设置提醒'
    case 'admin':
      return '管理员自启动提醒'
    default:
      return '提醒'
  }
})

const dialogMessage = computed(() => {
  switch (reminderType.value) {
    case 'startup':
      return '检测到您还没有设置应用自启动。启用自启动后，应用将在系统启动时自动运行您配置的启动项。'
    case 'admin':
      return '检测到您有启动项需要管理员权限运行，但没有设置管理员自启动。建议启用管理员自启动以确保这些启动项能正常运行。'
    default:
      return ''
  }
})

const actionButtonText = computed(() => {
  switch (reminderType.value) {
    case 'startup':
      return '启用自启动'
    case 'admin':
      return '启用管理员自启动'
    default:
      return '确定'
  }
})

// 等待窗口完全加载
const waitForWindow = async () => {
  return new Promise<void>((resolve) => {
    const checkWindow = () => {
      if (document.readyState === 'complete' && window.innerWidth > 0) {
        windowReady.value = true
        resolve()
      } else {
        setTimeout(checkWindow, 100)
      }
    }
    checkWindow()
  })
}

// 检查是否需要显示提醒
const checkReminders = async () => {
  try {
    // 等待窗口完全加载
    await waitForWindow()
    
    // 额外延迟确保页面渲染完成
    await new Promise(resolve => setTimeout(resolve, 600))
    
    const [showStartupReminder, showAdminReminder] = await invoke<[boolean, boolean]>('check_startup_reminders')
    
    if (showStartupReminder) {
      reminderType.value = 'startup'
      // 延迟显示弹窗以产生更好的视觉效果
      setTimeout(() => {
        showDialog.value = true
      }, 500)
    } else if (showAdminReminder) {
      reminderType.value = 'admin'
      setTimeout(() => {
        showDialog.value = true
      }, 500)
    }
  } catch (error) {
    console.error('检查提醒状态失败:', error)
  }
}

// 启用自启动
const enableStartup = async () => {
  try {
    // 加载当前设置
    const settings = await invoke<any>('load_app_settings')
    
    if (reminderType.value === 'startup') {
      settings.autoStartupEnabled = true
      settings.autoStartupAsAdmin = false
    } else if (reminderType.value === 'admin') {
      settings.autoStartupEnabled = true
      settings.autoStartupAsAdmin = true
    }
    
    // 保存设置
    await invoke('save_app_settings', { settings })
    
    // 应用自启动设置
    await invoke('apply_startup_settings', { settings })
    
    emit('startupEnabled')
    closeDialog()
  } catch (error) {
    console.error('启用自启动失败:', error)
    alert('启用自启动失败: ' + error)
  }
}

// 关闭对话框
const closeDialog = async () => {
  if (dontShowAgain.value && reminderType.value) {
    try {
      const hideStartup = reminderType.value === 'startup'
      const hideAdmin = reminderType.value === 'admin'
      
      await invoke('update_reminder_settings', {
        hideStartup,
        hideAdmin
      })
    } catch (error) {
      console.error('更新提醒设置失败:', error)
    }
  }
  
  showDialog.value = false
  reminderType.value = null
  dontShowAgain.value = false
  emit('close')
}

// 暴露检查提醒的方法
defineExpose({
  checkReminders
})

onMounted(() => {
  if (props.show) {
    checkReminders()
  }
})
</script>

<style scoped>
.reminder-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.reminder-dialog {
  background: white;
  border-radius: 12px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  max-width: 480px;
  width: 90%;
  max-height: 90vh;
  overflow: hidden;
}

.reminder-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.reminder-header h3 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: #111827;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: #6b7280;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: #f3f4f6;
  color: #374151;
}

.reminder-content {
  padding: 24px;
}

.reminder-icon {
  display: flex;
  justify-content: center;
  margin-bottom: 16px;
}

.reminder-message {
  text-align: center;
  color: #374151;
  font-size: 1rem;
  line-height: 1.6;
  margin: 0 0 24px 0;
}

.reminder-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.btn {
  flex: 1;
  padding: 12px 16px;
  border: none;
  border-radius: 8px;
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover {
  background-color: #2563eb;
}

.btn-secondary {
  background-color: #f3f4f6;
  color: #374151;
  border: 1px solid #d1d5db;
}

.btn-secondary:hover {
  background-color: #e5e7eb;
}

.reminder-checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
}

.reminder-checkbox label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
  color: #6b7280;
  cursor: pointer;
}

.reminder-checkbox input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: #3b82f6;
}

/* 动画效果 */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.slide-up-enter-active {
  transition: all 0.6s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.slide-up-leave-active {
  transition: all 0.4s cubic-bezier(0.55, 0.055, 0.675, 0.19);
}

.slide-up-enter-from {
  opacity: 0;
  transform: translateY(50px) scale(0.9);
}

.slide-up-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(1.05);
}

.reminder-dialog {
  transform-origin: center;
}
</style>