// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//モジュール宣言
mod tray;
mod config;
mod types;
mod nas_monitor;

use tauri::menu::MenuBuilder;
use tauri::Manager;

use tauri_plugin_dialog::{DialogExt,MessageDialogKind};
use tauri_plugin_log::{fern, Target, TargetKind};
use tauri_plugin_single_instance::init as single_instance;

use config::init_initial_info;
use nas_monitor::NasMonitor;
use crate::types::NasConfig;
use tauri::{command, State};

/// NASの現在の状態を取得
#[command]
async fn get_nas_status(monitor: State<'_, NasMonitor>) -> Result<Vec<NasConfig>, String> {
    Ok(monitor.get_configs().await)
}

/// NASの転送状態を更新
#[command]
async fn set_nas_transfer_status(
    monitor: State<'_, NasMonitor>,
    nas_id: u32,
    is_transfer: bool
) -> Result<(), String> {
    monitor.set_transfer_status(nas_id, is_transfer).await
}

fn main() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        init_initial_info,
        get_nas_status,
        set_nas_transfer_status
    ])
    .plugin(tauri_plugin_dialog::init())
    .plugin(single_instance(|app, _args, _cwd| {
        // 既にインスタンスが起動している場合、ウィンドウを表示
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }))
    .setup(|app|{
        // 初期設定を読み込んでNAS監視を開始
        let app_handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            match init_initial_info().await {
                Ok((nas_configs, _insp_configs)) => {
                    // NAS監視を開始
                    let monitor = NasMonitor::new(nas_configs);

                    // グローバル状態として管理
                    app_handle.manage(monitor.clone());

                    // 監視スレッドを開始
                    monitor.start_monitoring(app_handle.clone());

                    println!("NAS monitoring started successfully");
                }
                Err(e) => {
                    eprintln!("Failed to initialize NAS monitoring: {}", e);
                }
            }
        });

        //トレイアイコンをセットアップ
        tray::setup_tray_icon(app)?;

        //メニューバーを追加
        let menu = MenuBuilder::new(app)
            .text("version", "Version")
            .build()?;

        app.set_menu(menu)?;

        app.on_menu_event(|app_handle, event| {
            match event.id().as_ref() {
                "version" => {
                    let app_handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        app_handle.dialog()
                            .message("バージョン:0.0.1\n作成者:Takahashi Naoki")
                            .kind(MessageDialogKind::Info)
                            .title("バージョン情報")
                            .blocking_show();
                    });
                },
                _ => {
                    println!("unexpected menu event");
                }
            }
        });

        Ok(())

    })
    .on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        let _ = window.hide();
        api.prevent_close();
    }
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
