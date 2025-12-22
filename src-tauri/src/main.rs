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
use tauri_plugin_log::{fern, Target, TargetKind};
use tauri_plugin_single_instance::init as single_instance;

use config::{init_info, save_settings, save_insp_settings, save_nas_settings,save_insp_backup_setting};
use app_monitor::AppMonitor;
use settings_monitor::SettingsMonitor;
use backup_scheduler::BackupScheduler;
use crate::types::{NasConfig, InspConfig, SettingsConfig, BackupStatus,InspInfo,NasInfo};
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

/// 外観の設定を更新(メモリとファイルの両方)
#[command]
async fn edit_insp_configs(
    app_monitor: State<'_, AppMonitor>,
    scheduler: State<'_, BackupScheduler>,
    new_insp_info:InspInfo
) -> Result<Vec<InspConfig>, String> {
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // 先にメモリ上の設定を更新
    app_monitor.update_insp_configs(&new_insp_info).await;

    // メモリ上の更新が成功したらメモリの内容をファイルに保存
    save_insp_settings(new_insp_info,"edit").await?;

    //バックエンドとフロントエンドの状況の乖離が生じないように最新のinsp_configsを取得
    let insp_configs=app_monitor.get_insp_configs().await;

    Ok(insp_configs)
}

/// NASの設定を更新(メモリとファイルの両方)
#[command]
async fn edit_nas_configs(
    app_monitor: State<'_, AppMonitor>,
    scheduler: State<'_, BackupScheduler>,
    new_nas_info:NasInfo
) -> Result<Vec<NasConfig>, String> {
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // 先にメモリ上の設定を更新
    app_monitor.update_nas_configs(&new_nas_info).await;

    // メモリ上の更新が成功したらメモリの内容をファイルに保存
    save_nas_settings(new_nas_info,"edit").await?;

    //バックエンドとフロントエンドの状況の乖離が生じないように最新のinsp_configsを取得
    let nas_configs=app_monitor.get_nas_configs().await;

    log::debug!("{:?}",nas_configs);

    Ok(nas_configs)
}

//外観のis_backup切り替え
#[command]
async fn change_insp_backup_settings(
    app_monitor: State<'_, AppMonitor>,
    scheduler: State<'_, BackupScheduler>,
    insp_id:u32
) -> Result<Vec<InspConfig>, String> {
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // メモリ上の設定を更新
    app_monitor.switch_insp_backup_settings(insp_id).await;

    // 更新後の状態を取得してファイルに保存
    let insp_configs = app_monitor.get_insp_configs().await;

    // 該当IDのis_backup状態を取得
    if let Some(insp) = insp_configs.iter().find(|c| c.id == insp_id) {
        save_insp_backup_setting(insp_id, insp.is_backup).await?;
    }

    Ok(insp_configs)
}

//外観検査機器の追加
#[command]
async fn add_insp_configs(
    app_monitor: State<'_, AppMonitor>,
    scheduler: State<'_, BackupScheduler>,
    name:String,
    insp_ip:String,
    surface_image_path:String,
    back_image_path:String,
    surface_result_path:String,
    back_result_path:String
) -> Result<Vec<InspConfig>, String> {
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // メモリ上の設定を更新
    let new_id=app_monitor.add_insp(name.clone(),insp_ip.clone(),surface_image_path.clone(),back_image_path.clone(),surface_result_path.clone(),back_result_path.clone()).await;

    // 更新後のメモリ上の設定を取得
    let insp_configs = app_monitor.get_insp_configs().await;

    // メモリ上の更新が成功したらメモリの内容をファイルに保存
    //save_insp_settingsに渡すためにInspInfoを作成
    let add_insp_info:InspInfo=InspInfo { id: new_id, name, insp_ip, surface_image_path, back_image_path, surface_result_path,back_result_path, is_backup:true };
    save_insp_settings(add_insp_info,"add").await?;

    log::debug!("{:?}",insp_configs);

    Ok(insp_configs)
}

//NASの追加
#[command]
async fn add_nas_configs(
    app_monitor: State<'_, AppMonitor>,
    scheduler: State<'_, BackupScheduler>,
    name:String,
    nas_ip:String,
    drive:String,
) -> Result<Vec<NasConfig>, String> {
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // メモリ上の設定を更新
    let new_id=app_monitor.add_nas(name.clone(),nas_ip.clone(),drive.clone()).await;

    // 更新後のメモリ上の設定を取得
    let nas_configs = app_monitor.get_nas_configs().await;

    // メモリ上の更新が成功したらメモリの内容をファイルに保存
    //save_insp_settingsに渡すためにInspInfoを作成
    let add_nas_info:NasInfo=NasInfo { id: new_id, name, nas_ip,drive};
    save_nas_settings(add_nas_info,"add").await?;

    log::debug!("{:?}",nas_configs);

    Ok(nas_configs)
}


///外観検査機器設定の削除を実施
#[command]
async fn delete_insp_configs(
    app_monitor: State<'_, AppMonitor>,
    scheduler: State<'_, BackupScheduler>,
    id:u32,
)->Result<Vec<InspConfig>,String>{
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // 先にメモリ上の設定を更新
    let deleted_insp_info = app_monitor.delete_insp(id).await
        .ok_or("指定されたIDの検査機器が見つかりませんでした".to_string())?;

    // メモリ上の更新が成功したらメモリの内容をファイルに保存
    save_insp_settings(deleted_insp_info,"delete").await?;

    //バックエンドとフロントエンドの状況の乖離が生じないように最新のinsp_configsを取得
    let insp_configs=app_monitor.get_insp_configs().await;

    log::debug!("{:?}",insp_configs);

    Ok(insp_configs)

}

///NASの削除を実施
#[command]
async fn delete_nas_configs(
    app_monitor: State<'_, AppMonitor>,
    scheduler: State<'_, BackupScheduler>,
    id:u32,
)->Result<Vec<NasConfig>,String>{
    // バックアップ中は設定変更を拒否
    if scheduler.is_backup_running().await {
        return Err("バックアップ実行中は設定を変更できません".to_string());
    }

    // 先にメモリ上の設定を更新
    let deleted_nas_info = app_monitor.delete_nas(id).await
        .ok_or("指定されたIDの検査機器が見つかりませんでした".to_string())?;

    // メモリ上の更新が成功したらメモリの内容をファイルに保存
    save_nas_settings(deleted_nas_info,"delete").await?;

    //バックエンドとフロントエンドの状況の乖離が生じないように最新のinsp_configsを取得
    let nas_configs=app_monitor.get_nas_configs().await;

    log::debug!("{:?}",nas_configs);

    Ok(nas_configs)

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
        edit_insp_configs,
        edit_nas_configs,
        change_insp_backup_settings,
        add_insp_configs,
        add_nas_configs,
        delete_insp_configs,
        delete_nas_configs,
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
    .plugin(
        tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::Dispatch(
                    fern::Dispatch::new().chain(
                        fern::DateBased::new("logs/", "%Y-%m-%d.log")
                    )
                )),
            ])
            .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
            .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
            .level(log::LevelFilter::Info)
            .build(),
    )
    .setup(|app|{
        // ログディレクトリを作成
        if let Err(e) = std::fs::create_dir_all("logs") {
            log::error!("Failed to create logs directory: {}", e);
        }

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
                    ).await;

                    // グローバル状態として管理
                    app_handle.manage(app_monitor.clone());
                    app_handle.manage(settings_monitor.clone());
                    app_handle.manage(backup_scheduler.clone());

                    // 監視スレッドを開始
                    app_monitor.start_monitoring(app_handle.clone());

                    // バックアップスケジューラを開始
                    backup_scheduler.start_scheduling(app_handle.clone());

                    log::info!("Application monitoring and backup scheduler started successfully");
                }
                Err(e) => {
                    log::error!("Failed to initialize application monitoring: {}", e);
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
                    log::warn!("unexpected menu event");
                }
            }
        });

        log::info!("アプリを起動しました");
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
