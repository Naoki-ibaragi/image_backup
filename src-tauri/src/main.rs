// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//モジュール宣言
mod tray;
mod config;
mod types;
mod app_monitor;

use tauri::menu::MenuBuilder;
use tauri::Manager;

use tauri_plugin_dialog::{DialogExt,MessageDialogKind};
use tauri_plugin_single_instance::init as single_instance;

use config::init_info;
use app_monitor::AppMonitor;
use crate::types::{NasConfig, InspConfig,TransferState};
use tauri::{command, State};

/// NASの現在の状態を取得
#[command]
async fn get_nas_status(monitor: State<'_, AppMonitor>) -> Result<Vec<NasConfig>, String> {
    Ok(monitor.get_nas_configs().await)
}

/// 検査機器の現在の状態を取得
#[command]
async fn get_insp_status(monitor: State<'_, AppMonitor>) -> Result<Vec<InspConfig>, String> {
    Ok(monitor.get_insp_configs().await)
}

fn main() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        init_info, //NAS情報と外観情報の初期化
        get_nas_status,
        get_insp_status,
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
        // 初期設定を読み込んでアプリケーション監視を開始
        let app_handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            match init_info().await {
                Ok(configs) => {
                    // アプリケーション監視を開始（NASと検査機器の両方）
                    let app_monitor = AppMonitor::new(configs.nas_configs, configs.insp_configs);

                    // グローバル状態として管理
                    app_handle.manage(app_monitor.clone());

                    // 監視スレッドを開始
                    app_monitor.start_monitoring(app_handle.clone());

                    let mut a: Option<u32>;
                    let mut b: Option<u32>;
                    let mut c: Option<u32>;
                    let mut d: Option<u32>;

                    match app_monitor.nas_configs.len(){
                        2 => {
                            a = Some(1);
                            b = Some(2);
                        }
                        1 => {
                            a = Some(1);
                            b = None;
                        }
                        _ => {
                            a = None;
                            b = None;
                        }
                    }

                    match app_monitor.insp_configs.len(){
                        2 => {
                            c = Some(1);
                            d = Some(2);
                        }
                        1 => {
                            c = Some(1);
                            d = None;
                        }
                        _ => {
                            c = None;
                            d = None;
                        }
                    }

                    let transfer_state = TransferState::new(a,b,c,d);
                    app_handle.manage(transfer_state);

                    println!("Application monitoring started successfully");
                }
                Err(e) => {
                    eprintln!("Failed to initialize application monitoring: {}", e);
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
