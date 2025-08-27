use tauri::{Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder, AppHandle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub mode: String, // "normal" or "command"
    pub executable_path: String,
    pub display_name: String,
    pub icon: String,
    pub arguments: String,
    pub command: String,
    pub run_as_admin: bool,
    pub enabled: bool,
    pub delay_enabled: bool,
    pub delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub auto_startup_enabled: bool,
    pub auto_startup_as_admin: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub check_updates: bool,
    pub hide_startup_reminder: bool,
    pub hide_admin_startup_reminder: bool,
    pub exit_after_startup: bool,
}

// 获取数据目录路径
fn get_data_dir(_app: &AppHandle) -> Result<PathBuf, String> {
    // 获取可执行文件路径
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    // 获取可执行文件所在目录
    let exe_dir = exe_path.parent()
        .ok_or("Failed to get executable directory")?;
    
    // 创建data子目录路径
    let data_dir = exe_dir.join("data");
    
    // 确保data目录存在
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }
    
    Ok(data_dir)
}

// 获取启动项配置文件路径
fn get_startup_items_file(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_data_dir(app)?;
    Ok(data_dir.join("startup_items.json"))
}

// 获取应用设置配置文件路径
fn get_app_settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_data_dir(app)?;
    Ok(data_dir.join("app_settings.json"))
}

// 默认应用设置
fn default_app_settings() -> AppSettings {
    AppSettings {
        auto_startup_enabled: false,
        auto_startup_as_admin: false,
        minimize_to_tray: true,
        start_minimized: false,
        check_updates: true,
        hide_startup_reminder: false,
        hide_admin_startup_reminder: false,
        exit_after_startup: false,
    }
}

// 获取当前可执行文件路径
fn get_current_exe_path() -> Result<String, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
        .to_string_lossy()
        .to_string();
    Ok(exe_path)
}

// 设置普通自启动（注册表方式）
fn set_normal_startup(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_SET_VALUE | KEY_QUERY_VALUE)
        .map_err(|e| format!("打开注册表Run键失败: {}", e))?;
    
    let app_name = "EasiStartup";
    
    if enabled {
        let exe_path = get_current_exe_path()?;
        let startup_command = format!("\"{}\" --auto", exe_path);
        run_key
            .set_value(app_name, &startup_command)
            .map_err(|e| format!("设置注册表自启动失败: {}", e))?;
    } else {
        // 删除注册表项（忽略不存在的错误）
        let _ = run_key.delete_value(app_name);
    }
    
    Ok(())
}

// 检查普通自启动状态
#[tauri::command]
fn check_normal_startup() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| format!("打开注册表Run键失败: {}", e))?;
    
    let app_name = "EasiStartup";
    match run_key.get_value::<String, _>(app_name) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// 设置管理员自启动（计划任务方式）
fn set_admin_startup(enabled: bool) -> Result<(), String> {
    let task_name = "EasiStartup_AdminTask";
    
    if enabled {
        let exe_path = get_current_exe_path()?;
        
        // 创建计划任务的XML配置
        let task_xml = format!(r#"
<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>EasiStartup 自启动任务</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>true</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{}</Command>
      <Arguments>--auto</Arguments>
    </Exec>
  </Actions>
</Task>
"#, exe_path);
        
        // 使用schtasks命令创建任务
        let mut cmd = Command::new("schtasks");
        cmd.args(["/create", "/tn", task_name, "/xml", "-", "/f"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        
        // 在Windows上隐藏命令行窗口
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let output = cmd.spawn()
            .map_err(|e| format!("启动schtasks命令失败: {}", e))?;
        
        if let Some(mut stdin) = output.stdin.as_ref() {
            use std::io::Write;
            stdin.write_all(task_xml.as_bytes())
                .map_err(|e| format!("写入任务XML失败: {}", e))?;
        }
        
        let result = output.wait_with_output()
            .map_err(|e| format!("等待schtasks命令完成失败: {}", e))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("创建计划任务失败: {}", stderr));
        }
    } else {
        // 删除计划任务
        let mut cmd = Command::new("schtasks");
        cmd.args(["/delete", "/tn", task_name, "/f"]);
        
        // 在Windows上隐藏命令行窗口
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let output = cmd.output()
            .map_err(|e| format!("删除计划任务失败: {}", e))?;
        
        // 忽略任务不存在的错误
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("cannot find the file") && !stderr.contains("找不到") {
                return Err(format!("删除计划任务失败: {}", stderr));
            }
        }
    }
    
    Ok(())
}

// 检查管理员自启动状态
#[tauri::command]
fn check_admin_startup() -> Result<bool, String> {
    let task_name = "EasiStartup_AdminTask";
    
    let mut cmd = Command::new("schtasks");
    cmd.args(["/query", "/tn", task_name]);
    
    // 在Windows上隐藏命令行窗口
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    let output = cmd.output()
        .map_err(|e| format!("查询计划任务失败: {}", e))?;
    
    Ok(output.status.success())
}

// 检查是否以管理员身份运行
fn is_running_as_admin() -> Result<bool, String> {
    // 使用Windows API直接检查，避免PowerShell调用
    #[cfg(windows)]
    {
        use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::{TOKEN_QUERY, TokenElevation, TOKEN_ELEVATION};
        use std::mem;
        use std::ptr;
        
        unsafe {
            let mut token_handle = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) == 0 {
                return Ok(false);
            }
            
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0;
            
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );
            
            winapi::um::handleapi::CloseHandle(token_handle);
            
            if result != 0 {
                Ok(elevation.TokenIsElevated != 0)
            } else {
                Ok(false)
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

// 以管理员身份重启应用
fn restart_as_admin() -> Result<(), String> {
    let exe_path = get_current_exe_path()?;
    
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        
        // 使用ShellExecuteW API直接启动，避免PowerShell
        unsafe {
            let exe_path_wide: Vec<u16> = OsStr::new(&exe_path).encode_wide().chain(std::iter::once(0)).collect();
            let verb_wide: Vec<u16> = OsStr::new("runas").encode_wide().chain(std::iter::once(0)).collect();
            let params_wide: Vec<u16> = OsStr::new("--auto").encode_wide().chain(std::iter::once(0)).collect();
            
            let result = winapi::um::shellapi::ShellExecuteW(
                std::ptr::null_mut(),
                verb_wide.as_ptr(),
                exe_path_wide.as_ptr(),
                params_wide.as_ptr(),
                std::ptr::null(),
                winapi::um::winuser::SW_HIDE,
            );
            
            if result as isize <= 32 {
                return Err("以管理员身份重启失败".to_string());
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        return Err("非Windows系统不支持管理员重启".to_string());
    }
    
    // 退出当前进程
    std::process::exit(0);
}

// 加载应用设置
#[tauri::command]
fn load_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    let settings_file = get_app_settings_file(&app)?;
    
    if !settings_file.exists() {
        return Ok(default_app_settings());
    }
    
    let content = std::fs::read_to_string(&settings_file)
        .map_err(|e| format!("读取设置文件失败: {}", e))?;
    
    let settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("解析设置文件失败: {}", e))?;
    
    Ok(settings)
}

// 保存应用设置
#[tauri::command]
fn save_app_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let settings_file = get_app_settings_file(&app)?;
    
    // 确保父目录存在
    if let Some(parent) = settings_file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建设置目录失败: {}", e))?;
    }
    
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;
    
    std::fs::write(&settings_file, content)
        .map_err(|e| format!("写入设置文件失败: {}", e))?;
    
    Ok(())
}

// 应用自启动设置
#[tauri::command]
fn apply_startup_settings(settings: AppSettings) -> Result<(), String> {
    if settings.auto_startup_enabled {
        if settings.auto_startup_as_admin {
            // 需要管理员权限的自启动
            if !is_running_as_admin()? {
                return Err("需要管理员权限来设置管理员自启动".to_string());
            }
            
            // 删除普通自启动
            set_normal_startup(false)?;
            // 设置管理员自启动
            set_admin_startup(true)?;
        } else {
            // 普通自启动
            set_admin_startup(false)?;
            set_normal_startup(true)?;
        }
    } else {
        // 禁用所有自启动
        set_normal_startup(false)?;
        set_admin_startup(false)?;
    }
    
    Ok(())
}

// 检查当前权限状态
#[tauri::command]
fn check_admin_permission() -> Result<bool, String> {
    is_running_as_admin()
}

// 请求管理员权限重启
#[tauri::command]
fn request_admin_restart() -> Result<(), String> {
    restart_as_admin()
}

// 检查启动项中是否有需要管理员权限的项目
fn has_admin_startup_items(app: &AppHandle) -> Result<bool, String> {
    let file_path = get_startup_items_file(app)?;
    
    if !file_path.exists() {
        return Ok(false);
    }
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取启动项文件失败: {}", e))?;
    
    let items: Vec<StartupItem> = serde_json::from_str(&content)
        .map_err(|e| format!("解析启动项失败: {}", e))?;
    
    // 检查是否有启用的管理员启动项
    Ok(items.iter().any(|item| item.enabled && item.run_as_admin))
}

// 检查是否需要显示自启动提醒
#[tauri::command]
fn check_startup_reminders(app: AppHandle) -> Result<(bool, bool), String> {
    let settings = load_app_settings(app.clone())?;
    
    // 如果用户已经选择不再显示提醒，直接返回false
    if settings.hide_startup_reminder && settings.hide_admin_startup_reminder {
        return Ok((false, false));
    }
    
    let normal_startup_enabled = check_normal_startup()?;
    let admin_startup_enabled = check_admin_startup()?;
    let has_admin_items = has_admin_startup_items(&app)?;
    
    // 检查是否需要显示普通自启动提醒
    let show_startup_reminder = !settings.hide_startup_reminder && 
        !normal_startup_enabled && !admin_startup_enabled;
    
    // 检查是否需要显示管理员自启动提醒
    let show_admin_reminder = !settings.hide_admin_startup_reminder && 
        has_admin_items && !admin_startup_enabled;
    
    Ok((show_startup_reminder, show_admin_reminder))
}

// 更新提醒设置
#[tauri::command]
fn update_reminder_settings(app: AppHandle, hide_startup: bool, hide_admin: bool) -> Result<(), String> {
    let mut settings = load_app_settings(app.clone())?;
    settings.hide_startup_reminder = hide_startup;
    settings.hide_admin_startup_reminder = hide_admin;
    save_app_settings(app, settings)?;
    Ok(())
}

// 获取应用版本
#[tauri::command]
fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn open_file_dialog(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    
    app.dialog()
        .file()
        .add_filter("Executable files", &["exe"])
        .add_filter("All files", &["*"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });
    
    // 等待回调结果
    match rx.recv() {
        Ok(Some(path)) => Ok(Some(path.to_string())),
        Ok(None) => Ok(None),
        Err(_) => Err("Failed to receive file dialog result".to_string()),
    }
}

#[tauri::command]
async fn load_startup_items(app: AppHandle) -> Result<Vec<StartupItem>, String> {
    let file_path = get_startup_items_file(&app)?;
    
    if !file_path.exists() {
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read startup items file: {}", e))?;
    
    let items: Vec<StartupItem> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse startup items: {}", e))?;
    
    Ok(items)
}

#[tauri::command]
async fn save_startup_items(app: AppHandle, items: Vec<StartupItem>) -> Result<(), String> {
    let file_path = get_startup_items_file(&app)?;
    
    let content = serde_json::to_string_pretty(&items)
        .map_err(|e| format!("Failed to serialize startup items: {}", e))?;
    
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write startup items file: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn create_startup_item() -> StartupItem {
    StartupItem {
        id: Uuid::new_v4().to_string(),
        name: String::new(),
        mode: "normal".to_string(),
        executable_path: String::new(),
        display_name: String::new(),
        icon: String::new(),
        arguments: String::new(),
        command: String::new(),
        run_as_admin: false,
        enabled: true,
        delay_enabled: false,
        delay_seconds: 5,
    }
}

#[tauri::command]
fn show_startup_editor(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("startup-editor") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn show_settings(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
async fn get_shortcut_info(executable_path: String) -> Result<Option<(String, String)>, String> {
    // 检查是否是快捷方式文件
    if !executable_path.to_lowercase().ends_with(".lnk") {
        return Ok(None);
    }
    
    // 使用Windows API直接读取快捷方式，避免PowerShell
    // 暂时简化处理，避免复杂的COM操作
    // 对于.lnk文件，直接返回None让前端使用默认处理
    #[cfg(windows)]
    {
        // 检查是否是快捷方式文件
        if executable_path.to_lowercase().ends_with(".lnk") {
            // 从文件名获取显示名称
            if let Some(file_name) = std::path::Path::new(&executable_path).file_stem() {
                let display_name = file_name.to_string_lossy().to_string();
                // 返回原路径和显示名称，让前端处理
                return Ok(Some((executable_path, display_name)));
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        return Ok(None);
    }
    
    Ok(None)
}

// 执行单个启动项
#[tauri::command]
async fn execute_startup_item(item: StartupItem) -> Result<(), String> {
    if !item.enabled {
        return Ok(());
    }

    // 如果启用了延迟，先等待
    if item.delay_enabled && item.delay_seconds > 0 {
        thread::sleep(Duration::from_secs(item.delay_seconds as u64));
    }

    match item.mode.as_str() {
        "normal" => {
            if item.executable_path.is_empty() {
                return Err("可执行文件路径为空".to_string());
            }

            if item.run_as_admin {
                // 以管理员身份运行 - 使用Windows API直接启动，避免PowerShell
                #[cfg(windows)]
                {
                    use std::ffi::OsStr;
                    use std::os::windows::ffi::OsStrExt;
                    
                    unsafe {
                        let exe_path_wide: Vec<u16> = OsStr::new(&item.executable_path).encode_wide().chain(std::iter::once(0)).collect();
                        let verb_wide: Vec<u16> = OsStr::new("runas").encode_wide().chain(std::iter::once(0)).collect();
                        
                        let params_wide: Vec<u16> = if item.arguments.is_empty() {
                            vec![0]
                        } else {
                            OsStr::new(&item.arguments).encode_wide().chain(std::iter::once(0)).collect()
                        };
                        
                        let result = winapi::um::shellapi::ShellExecuteW(
                            std::ptr::null_mut(),
                            verb_wide.as_ptr(),
                            exe_path_wide.as_ptr(),
                            if params_wide.len() > 1 { params_wide.as_ptr() } else { std::ptr::null() },
                            std::ptr::null(),
                            winapi::um::winuser::SW_HIDE,
                        );
                        
                        if result as isize <= 32 {
                            return Err(format!("以管理员身份启动程序失败: {}", item.executable_path));
                        }
                    }
                }
                
                #[cfg(not(windows))]
                {
                    return Err("非Windows系统不支持管理员权限启动".to_string());
                }
            } else {
                // 普通运行 - 直接启动，不使用PowerShell
                let mut normal_cmd = Command::new(&item.executable_path);
                if !item.arguments.is_empty() {
                    // 简单的参数分割，可能需要更复杂的解析
                    let args: Vec<&str> = item.arguments.split_whitespace().collect();
                    normal_cmd.args(args);
                }
                
                // 在Windows上隐藏命令行窗口（如果启动的是控制台程序）
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    normal_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                }
                
                normal_cmd.spawn()
                    .map_err(|e| format!("启动程序失败: {}", e))?;
            }


        }
        "command" => {
            if item.command.is_empty() {
                return Err("命令为空".to_string());
            }

            let mut cmd = if item.run_as_admin {
                // 以管理员身份运行命令
                let mut admin_cmd = Command::new("powershell");
                admin_cmd.args([
                    "-ExecutionPolicy", "Bypass",
                    "-Command",
                    &format!("Start-Process powershell -ArgumentList @('-ExecutionPolicy', 'Bypass', '-Command', '{}') -Verb RunAs -ErrorAction Stop", 
                        item.command.replace("'", "''"))
                ]);
                
                // 在Windows上隐藏命令行窗口
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    admin_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                }
                
                admin_cmd
            } else {
                // 普通运行命令
                let mut normal_cmd = Command::new("powershell");
                normal_cmd.args(["-ExecutionPolicy", "Bypass", "-Command", &item.command]);
                
                // 在Windows上隐藏命令行窗口
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    normal_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                }
                
                normal_cmd
            };

            cmd.spawn()
                .map_err(|e| format!("执行命令失败: {}", e))?;
        }
        _ => {
            return Err(format!("未知的启动项模式: {}", item.mode));
        }
    }

    Ok(())
}

// 执行所有启动项
#[tauri::command]
async fn execute_all_startup_items(app: AppHandle) -> Result<(), String> {
    let items = load_startup_items(app.clone()).await?;
    
    for item in items {
        if item.enabled {
            if let Err(e) = execute_startup_item(item.clone()).await {
                eprintln!("执行启动项 '{}' 失败: {}", item.name, e);
                // 继续执行其他启动项，不因为一个失败而停止
            }
        }
    }
    
    // 检查是否需要在执行完启动项后退出
    let settings = load_app_settings(app)?;
    if settings.exit_after_startup {
        // 等待一小段时间确保所有启动项都已启动
        thread::sleep(Duration::from_millis(1000));
        std::process::exit(0);
    }
    
    Ok(())
}

#[tauri::command]
async fn get_executable_icon(app: AppHandle, executable_path: String) -> Result<Option<String>, String> {
    
    // 获取数据目录用于存储图标
    let data_dir = get_data_dir(&app)?;
    let icons_dir = data_dir.join("icons");
    
    // 确保图标目录存在
    if !icons_dir.exists() {
        fs::create_dir_all(&icons_dir)
            .map_err(|e| format!("Failed to create icons directory: {}", e))?;
    }
    
    // 生成图标文件名（基于可执行文件路径的哈希）
    let icon_filename = format!("{}.png", 
        executable_path.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_"));
    let icon_path = icons_dir.join(&icon_filename);
    
    // 如果图标已存在，直接返回路径
    if icon_path.exists() {
        return Ok(Some(icon_path.to_string_lossy().to_string()));
    }
    
    // 检查文件是否存在
    if !std::path::Path::new(&executable_path).exists() {
        return Err(format!("Executable file not found: {}", executable_path));
    }
    
    // 使用Windows API直接提取图标，避免PowerShell
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;
        
        unsafe {
            let exe_path_wide: Vec<u16> = OsStr::new(&executable_path).encode_wide().chain(std::iter::once(0)).collect();
            
            // 提取大图标
            let mut large_icon = ptr::null_mut();
            let mut small_icon = ptr::null_mut();
            
            let result = winapi::um::shellapi::ExtractIconExW(
                exe_path_wide.as_ptr(),
                0,
                &mut large_icon,
                &mut small_icon,
                1,
            );
            
            if result > 0 && !large_icon.is_null() {
                // 这里简化处理，直接返回None让前端使用默认图标
                // 完整的图标保存需要更复杂的GDI+操作
                winapi::um::winuser::DestroyIcon(large_icon);
                if !small_icon.is_null() {
                    winapi::um::winuser::DestroyIcon(small_icon);
                }
                // 为了简化，我们不保存图标文件，直接返回None
                return Ok(None);
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        return Ok(None);
    }
    
    Ok(None)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 检查命令行参数
    let args: Vec<String> = std::env::args().collect();
    let auto_mode = args.iter().any(|arg| arg == "--auto");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(move |app| {
            // 如果是自动模式，执行所有启动项然后退出
            if auto_mode {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = execute_all_startup_items(app_handle).await {
                        eprintln!("自动执行启动项失败: {}", e);
                    }
                    std::process::exit(0);
                });
                return Ok(());
            }
            
            // 创建托盘菜单
            let startup_editor = MenuItem::with_id(app, "startup_editor", "启动项编辑", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "应用设置", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&startup_editor, &settings, &quit])?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| {
                    let handle = app.app_handle();
                    match event.id.as_ref() {
                        "startup_editor" => {
                            if let Some(window) = handle.get_webview_window("startup-editor") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "settings" => {
                            if let Some(window) = handle.get_webview_window("settings") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })

                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    window.hide().unwrap();
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            show_startup_editor, 
            show_settings,
            open_file_dialog,
            load_startup_items,
            save_startup_items,
            create_startup_item,
            get_executable_icon,
            get_shortcut_info,
            execute_startup_item,
            execute_all_startup_items,
            load_app_settings,
            save_app_settings,
            apply_startup_settings,
            check_admin_permission,
            request_admin_restart,
            check_normal_startup,
            check_admin_startup,
            check_startup_reminders,
            update_reminder_settings,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
