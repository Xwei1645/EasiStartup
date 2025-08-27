import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, FrontendSettings } from '../types/settings';

/**
 * 将后端设置转换为前端设置格式
 */
export function mapBackendToFrontend(backendSettings: AppSettings): FrontendSettings {
  return {
    autoStart: backendSettings.auto_startup_enabled,
    runAsAdmin: backendSettings.auto_startup_as_admin,
    exitAfterStartup: backendSettings.exit_after_startup,
  };
}

/**
 * 将前端设置转换为后端设置格式
 */
export function mapFrontendToBackend(frontendSettings: FrontendSettings): AppSettings {
  return {
    auto_startup_enabled: frontendSettings.autoStart,
    auto_startup_as_admin: frontendSettings.runAsAdmin,
    exit_after_startup: frontendSettings.exitAfterStartup,
  };
}

/**
 * 加载应用设置
 */
export async function loadAppSettings(): Promise<AppSettings> {
  try {
    return await invoke('load_app_settings') as AppSettings;
  } catch (error) {
    console.error('加载设置失败:', error);
    throw error;
  }
}

/**
 * 保存应用设置
 */
export async function saveAppSettings(settings: AppSettings): Promise<void> {
  try {
    await invoke('save_app_settings', { settings });
  } catch (error) {
    console.error('保存设置失败:', error);
    throw error;
  }
}

/**
 * 应用启动设置
 */
export async function applyStartupSettings(settings: AppSettings): Promise<void> {
  try {
    await invoke('apply_startup_settings', { settings });
  } catch (error) {
    console.error('应用启动设置失败:', error);
    throw error;
  }
}