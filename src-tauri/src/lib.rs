use tauri::{Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder, AppHandle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

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
    use std::process::Command;
    
    // 检查是否是快捷方式文件
    if !executable_path.to_lowercase().ends_with(".lnk") {
        return Ok(None);
    }
    
    // 使用PowerShell获取快捷方式信息
    let powershell_script = format!(
        r#"
        try {{
            $shell = New-Object -ComObject WScript.Shell
            $shortcut = $shell.CreateShortcut('{}')
            $targetPath = $shortcut.TargetPath
            $description = $shortcut.Description
            $workingDirectory = $shortcut.WorkingDirectory
            
            # 如果没有描述，尝试从快捷方式文件名获取
            if ([string]::IsNullOrEmpty($description)) {{
                $fileName = [System.IO.Path]::GetFileNameWithoutExtension('{}')
                $description = $fileName
            }}
            
            Write-Output "$targetPath|$description"
        }} catch {{
            Write-Output "error: $($_.Exception.Message)"
        }}
        "#,
        executable_path.replace("'", "''"),
        executable_path.replace("'", "''")
    );
    
    let output = Command::new("powershell")
        .args(["-Command", &powershell_script])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let result = output_str.trim();
    
    if result.starts_with("error:") {
        return Err(result.to_string());
    }
    
    if let Some((target_path, display_name)) = result.split_once('|') {
        if !target_path.is_empty() {
            return Ok(Some((target_path.to_string(), display_name.to_string())));
        }
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

            let mut cmd = if item.run_as_admin {
                // 以管理员身份运行
                let mut admin_cmd = Command::new("powershell");
                let powershell_command = if item.arguments.is_empty() {
                    format!("Start-Process '{}' -Verb RunAs", 
                        item.executable_path.replace("'", "''"))
                } else {
                    format!("Start-Process '{}' -ArgumentList '{}' -Verb RunAs", 
                        item.executable_path.replace("'", "''"),
                        item.arguments.replace("'", "''"))
                };
                admin_cmd.args(["-Command", &powershell_command]);
                admin_cmd
            } else {
                // 普通运行
                let mut normal_cmd = Command::new(&item.executable_path);
                if !item.arguments.is_empty() {
                    // 简单的参数分割，可能需要更复杂的解析
                    let args: Vec<&str> = item.arguments.split_whitespace().collect();
                    normal_cmd.args(args);
                }
                normal_cmd
            };

            cmd.spawn()
                .map_err(|e| format!("启动程序失败: {}", e))?;
        }
        "command" => {
            if item.command.is_empty() {
                return Err("命令为空".to_string());
            }

            let mut cmd = if item.run_as_admin {
                // 以管理员身份运行命令
                let mut admin_cmd = Command::new("powershell");
                admin_cmd.args([
                    "-Command",
                    &format!("Start-Process powershell -ArgumentList '-Command', '{}' -Verb RunAs", 
                        item.command.replace("'", "''"))
                ]);
                admin_cmd
            } else {
                // 普通运行命令
                let mut normal_cmd = Command::new("powershell");
                normal_cmd.args(["-Command", &item.command]);
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
    let items = load_startup_items(app).await?;
    
    for item in items {
        if item.enabled {
            if let Err(e) = execute_startup_item(item.clone()).await {
                eprintln!("执行启动项 '{}' 失败: {}", item.name, e);
                // 继续执行其他启动项，不因为一个失败而停止
            }
        }
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
    
    // 使用PowerShell提取图标
    let powershell_script = format!(
        r#"
        try {{
            Add-Type -AssemblyName System.Drawing
            $execPath = '{}'
            $iconPath = '{}'
            
            if (-not (Test-Path $execPath)) {{
                Write-Output 'file_not_found'
                exit
            }}
            
            $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($execPath)
            if ($icon) {{
                $bitmap = $icon.ToBitmap()
                $bitmap.Save($iconPath, [System.Drawing.Imaging.ImageFormat]::Png)
                $bitmap.Dispose()
                $icon.Dispose()
                Write-Output 'success'
            }} else {{
                Write-Output 'no_icon'
            }}
        }} catch {{
            Write-Output "error: $($_.Exception.Message)"
        }}
        "#,
        executable_path.replace("'", "''"),
        icon_path.to_string_lossy().replace("'", "''")
    );
    
    let output = Command::new("powershell")
        .args(["-Command", &powershell_script])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let result = output_str.trim();
    
    match result {
        "success" if icon_path.exists() => {
            Ok(Some(icon_path.to_string_lossy().to_string()))
        }
        "file_not_found" => {
            Err(format!("Executable file not found: {}", executable_path))
        }
        "no_icon" => {
            Ok(None)
        }
        result if result.starts_with("error:") => {
            Err(format!("Icon extraction failed: {}", result))
        }
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                Err(format!("PowerShell error: {}", stderr))
            } else {
                Ok(None)
            }
        }
    }
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
            execute_all_startup_items
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
