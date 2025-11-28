// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//モジュール宣言
mod tray;
mod config;
mod types;
mod app_monitor;
mod settings_monitor;
mod backup_scheduler;
mod backup_executor;

use tauri::menu::MenuBuilder;
use tauri::Manager;

use tauri_plugin_dialog::{DialogExt,MessageDialogKind};
use tauri_plugin_single_instance::init as single_instance;

use config::{init_info, save_settings};
use app_monitor::AppMonitor;
use settings_monitor::SettingsMonitor;
use backup_scheduler::BackupScheduler;
use crate::types::{NasConfig, InspConfig, SettingsConfig, BackupStatus};
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

/// 設定を取得
#[command]
async fn get_settings(settings: State<'_, SettingsMonitor>) -> Result<SettingsConfig, String> {
    Ok(settings.get_settings().await)
}

/// 設定を更新（メモリとファイルの両方）
#[command]
async fn update_settings(
    settings: State<'_, SettingsMonitor>,
    scheduler: State<'_, BackupScheduler>,
    new_settings: SettingsConfig
) -> Result<(), String> {
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // ファイルに保存
    save_settings(new_settings.clone()).await?;

    // メモリ上の設定も更新
    settings.update_settings(new_settings).await;

    Ok(())
}

/// バックアップの状態を取得
#[command]
async fn get_backup_status(scheduler: State<'_, BackupScheduler>) -> Result<BackupStatus, String> {
    Ok(scheduler.get_status().await)
}

fn main() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        init_info, //NAS情報と外観情報の初期化
        get_nas_status,
        get_insp_status,
        get_settings,
        update_settings,
        get_backup_status,
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
                    let app_monitor = AppMonitor::new(configs.0.nas_configs, configs.0.insp_configs);
                    let settings_monitor = SettingsMonitor::new(configs.1);

                    // バックアップスケジューラを作成
                    let backup_scheduler = BackupScheduler::new(
                        settings_monitor.clone(),
                        app_monitor.clone()
                    );

                    // グローバル状態として管理
                    app_handle.manage(app_monitor.clone());
                    app_handle.manage(settings_monitor.clone());
                    app_handle.manage(backup_scheduler.clone());

                    // 監視スレッドを開始
                    app_monitor.start_monitoring(app_handle.clone());

                    // バックアップスケジューラを開始
                    backup_scheduler.start_scheduling(app_handle.clone());

                    println!("Application monitoring and backup scheduler started successfully");
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
