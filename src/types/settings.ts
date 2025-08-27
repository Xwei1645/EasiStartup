// 应用设置类型定义
export interface AppSettings {
  auto_startup_enabled: boolean;
  auto_startup_as_admin: boolean;
  exit_after_startup: boolean;
}

// 前端设置类型定义
export interface FrontendSettings {
  autoStart: boolean;
  runAsAdmin: boolean;
  exitAfterStartup: boolean;
}

// 启动项类型定义
export interface StartupItem {
  id: string;
  name: string;
  path: string;
  args: string;
  enabled: boolean;
  icon?: string;
}