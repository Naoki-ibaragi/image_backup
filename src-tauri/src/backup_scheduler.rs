use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tauri::{AppHandle, Emitter};
use chrono::{Local, Timelike};

use crate::app_monitor::AppMonitor;
use crate::settings_monitor::SettingsMonitor;
use crate::backup_executor::BackupExecutor;
use crate::types::BackupStatus;

/// バックアップのスケジューリングを担当する構造体
#[derive(Clone)]
pub struct BackupScheduler {
    settings_monitor: SettingsMonitor,
    app_monitor: AppMonitor,
    is_running: Arc<RwLock<bool>>,
    last_backup_date: Arc<RwLock<Option<String>>>,
    last_backup_nas_id: Arc<RwLock<Option<u32>>>,
}

impl BackupScheduler {
    /// 新しいBackupSchedulerインスタンスを作成
    pub async fn new(settings_monitor: SettingsMonitor, app_monitor: AppMonitor) -> Self {

        // 使用可能で接続されているNASのみをフィルタ
        let nas_configs = app_monitor.get_nas_configs().await;
        let nas_ids: Vec<u32> = nas_configs
            .iter()
            .filter_map(|nas| if nas.is_use && nas.is_connected {Some(nas.id)} else {None})
            .collect();

        let mut target_nas_id=None;
        if nas_ids.len()!=0{ target_nas_id=Some(nas_ids[0])};

        Self {
            settings_monitor,
            app_monitor,
            is_running: Arc::new(RwLock::new(false)),
            last_backup_date: Arc::new(RwLock::new(None)),
            last_backup_nas_id:Arc::new(RwLock::new(target_nas_id))
        }
    }

    /// スケジューリングを開始（毎分チェック）
    pub fn start_scheduling(self, app_handle: AppHandle) {
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // 1分ごと

            loop {
                interval.tick().await;

                // バックアップ実行中はスキップ
                if *self.is_running.read().await {
                    continue;
                }

                // 現在時刻を取得
                let now = Local::now();
                let current_time = format!("{:02}:{:02}", now.hour(), now.minute());
                let current_date = now.format("%Y-%m-%d").to_string();

                // 設定されたバックアップ時刻を取得
                let backup_time = self.settings_monitor.get_backup_time().await;

                // 今日既にバックアップ済みかチェック
                let last_backup = self.last_backup_date.read().await.clone();
                let already_backed_up_today = last_backup.as_ref() == Some(&current_date);

                // 時刻が一致し、今日未実行の場合にバックアップ開始
                //デバッグ時1日1回バックアップの制限外す
                //if current_time == backup_time && !already_backed_up_today { 
                if current_time == backup_time{
                    log::info!("Backup time reached: {} - Starting backup...", current_time);

                    if let Err(e) = self.execute_backup(app_handle.clone()).await {
                        let end_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        log::error!("Backup failed: {}", e);
                        let _ = app_handle.emit("backup-failed", (e,end_time));
                    }
                }
            }
        });
    }

    /// バックアップを実行
    async fn execute_backup(&self, app_handle: AppHandle) -> Result<(), String> {
        // 実行中フラグを立てる
        *self.is_running.write().await = true;

        log::info!("Starting backup execution...");

        // 開始イベントを通知
        let start_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = app_handle.emit("backup-started", start_time.clone());

        // NAS設定と検査機器設定を取得
        let nas_configs = self.app_monitor.get_nas_configs().await;
        let insp_configs = self.app_monitor.get_insp_configs().await;
        let settings = self.settings_monitor.get_settings().await;

        // バックアップを実行
        let result = BackupExecutor::execute(
            insp_configs,
            nas_configs,
            settings,
            app_handle.clone(),
        *self.last_backup_nas_id.read().await,
        ).await;

        // 実行中フラグを下ろす
        *self.is_running.write().await = false;

        match result {
            Ok(backup_result) => {
                // 成功した場合、最終バックアップ日を更新
                let current_date = Local::now().format("%Y-%m-%d").to_string();
                *self.last_backup_date.write().await = Some(current_date);

                let end_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                log::info!("Backup completed successfully: {:?}", backup_result);
                let _ = app_handle.emit("backup-completed", (backup_result,end_time));
                Ok(())
            }
            Err(e) => {
                log::error!("Backup execution error: {}", e);
                Err(e)
            }
        }
    }

    /// バックアップが実行中かどうかを取得
    pub async fn is_backup_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// 最後にバックアップした日付を取得
    pub async fn get_last_backup_date(&self) -> Option<String> {
        self.last_backup_date.read().await.clone()
    }

    /// バックアップステータスを取得
    pub async fn get_status(&self) -> BackupStatus {
        BackupStatus {
            is_running: self.is_backup_running().await,
            last_backup_date: self.get_last_backup_date().await,
        }
    }
}
