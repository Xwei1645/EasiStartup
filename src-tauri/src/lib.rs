use tauri::{Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {

            
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
        .invoke_handler(tauri::generate_handler![show_startup_editor, show_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
