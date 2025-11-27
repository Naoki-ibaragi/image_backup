use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tauri::{AppHandle, Emitter};
use sysinfo::Disks;
use std::net::TcpStream;
use std::time::Duration as StdDuration;
use crate::types::{NasConfig, InspConfig};

/// アプリケーション全体の状態を管理する構造体
/// NASと検査機器の両方の状態を一元管理
#[derive(Clone)]
pub struct AppMonitor {
    pub nas_configs: Arc<RwLock<Vec<NasConfig>>>,
    pub insp_configs: Arc<RwLock<Vec<InspConfig>>>,
}

impl AppMonitor {
    /// 新しいAppMonitorインスタンスを作成
    pub fn new(nas_configs: Vec<NasConfig>, insp_configs: Vec<InspConfig>) -> Self {
        Self {
            nas_configs: Arc::new(RwLock::new(nas_configs)),
            insp_configs: Arc::new(RwLock::new(insp_configs)),
        }
    }

    /// 監視スレッドを開始（10秒ごとにNASと検査機器の状態をチェック）
    pub fn start_monitoring(self, app_handle: AppHandle) {
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                // NAS状態を更新
                if let Err(e) = self.update_nas_status().await {
                    eprintln!("Failed to update NAS status: {}", e);
                }

                // 検査機器の状態を更新
                if let Err(e) = self.update_insp_status().await {
                    eprintln!("Failed to update inspection device status: {}", e);
                }

                // フロントエンドに更新を通知
                let nas_configs = self.nas_configs.read().await;
                if let Err(e) = app_handle.emit("nas-status-updated", nas_configs.clone()) {
                    eprintln!("Failed to emit nas-status-updated event: {}", e);
                }

                let insp_configs = self.insp_configs.read().await;
                if let Err(e) = app_handle.emit("insp-status-updated", insp_configs.clone()) {
                    eprintln!("Failed to emit insp-status-updated event: {}", e);
                }
            }
        });
    }

    /// すべてのNASの状態を更新
    async fn update_nas_status(&self) -> Result<(), String> {
        let mut configs = self.nas_configs.write().await;

        for config in configs.iter_mut() {
            // 接続チェック
            config.is_connected = check_nas_connection(&config.nas_ip);

            // 接続できている場合は容量情報を取得
            if config.is_connected {
                if let Ok(space_info) = get_drive_space_info(&config.drive) {
                    config.total_space = space_info.total;
                    config.used_space = space_info.used;
                    config.free_space = space_info.free;
                } else {
                    println!("Warning: Could not get space info for drive {}", config.drive);
                }
            } else {
                // 接続できていない場合は容量を0にリセット
                config.total_space = 0;
                config.used_space = 0;
                config.free_space = 0;
            }
        }

        Ok(())
    }

    /// すべての検査機器の状態を更新
    async fn update_insp_status(&self) -> Result<(), String> {
        let mut configs = self.insp_configs.write().await;

        for config in configs.iter_mut() {
            // 検査機器への接続チェック（ポート番号は適宜調整してください）
            // ここでは仮に445ポートを使用していますが、実際の検査機器のポートに変更してください
            let is_connected = check_device_connection(&config.insp_ip);

            // 接続状態に応じてバックアップフラグを更新
            // 切断されている場合は自動的にバックアップを停止
            if !is_connected && config.is_backup {
                println!("Warning: Inspection device {} is disconnected", config.name);
            }
        }

        Ok(())
    }

    /// 現在のNAS設定を取得
    pub async fn get_nas_configs(&self) -> Vec<NasConfig> {
        self.nas_configs.read().await.clone()
    }

    /// 現在の検査機器設定を取得
    pub async fn get_insp_configs(&self) -> Vec<InspConfig> {
        self.insp_configs.read().await.clone()
    }
}

/// NASへの接続をチェック（SMBポート445へのTCP接続を試行）
fn check_nas_connection(nas_ip: &str) -> bool {
    let address = format!("{}:445", nas_ip);

    match TcpStream::connect_timeout(
        &address.parse().unwrap(),
        StdDuration::from_secs(3)
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// 検査機器への接続をチェック
/// ポート番号は検査機器の仕様に応じて変更してください
fn check_device_connection(device_ip: &str) -> bool {
    // 仮に445ポートでチェック（実際のポート番号に変更してください）
    let address = format!("{}:445", device_ip);

    match TcpStream::connect_timeout(
        &address.parse().unwrap(),
        StdDuration::from_secs(3)
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// ドライブの容量情報
struct DriveSpaceInfo {
    total: u64,
    used: u64,
    free: u64,
}

/// ドライブの容量情報を取得
fn get_drive_space_info(drive_letter: &str) -> Result<DriveSpaceInfo, String> {
    let disks = Disks::new_with_refreshed_list();

    // ドライブレターを正規化（例: "Z:" -> "Z:\\"）
    let drive_path = if drive_letter.ends_with(":\\") {
        drive_letter.to_string()
    } else if drive_letter.ends_with(":") {
        format!("{}\\", drive_letter)
    } else {
        format!("{}:\\", drive_letter)
    };

    for disk in &disks {
        let mount_point = disk.mount_point().to_string_lossy();

        if mount_point.to_uppercase() == drive_path.to_uppercase() {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total.saturating_sub(free);

            return Ok(DriveSpaceInfo {
                total,
                used,
                free,
            });
        }
    }

    Err(format!("Drive {} not found", drive_path))
}

#[command]
fn get_transfer_info(transfer_info:)
