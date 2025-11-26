use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tauri::{AppHandle, Emitter};
use sysinfo::{System, Disks};
use std::net::TcpStream;
use std::time::Duration as StdDuration;
use crate::types::NasConfig;

/// グローバルなNAS状態を管理する構造体
#[derive(Clone)]
pub struct NasMonitor {
    pub nas_configs: Arc<RwLock<Vec<NasConfig>>>,
}

impl NasMonitor {
    /// 新しいNasMonitorインスタンスを作成
    pub fn new(initial_configs: Vec<NasConfig>) -> Self {
        Self {
            nas_configs: Arc::new(RwLock::new(initial_configs)),
        }
    }

    /// 監視スレッドを開始（10秒ごとにNASの状態をチェック）
    pub fn start_monitoring(self, app_handle: AppHandle) {
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                // NAS状態を更新
                if let Err(e) = self.update_nas_status().await {
                    eprintln!("Failed to update NAS status: {}", e);
                }

                // フロントエンドに更新を通知
                let configs = self.nas_configs.read().await;
                if let Err(e) = app_handle.emit("nas-status-updated", configs.clone()) {
                    eprintln!("Failed to emit nas-status-updated event: {}", e);
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
                    config.current_space = space_info.used;
                    config.free_space = space_info.free;
                } else {
                    println!("Warning: Could not get space info for drive {}", config.drive);
                }
            } else {
                // 接続できていない場合は容量を0にリセット
                config.total_space = 0;
                config.current_space = 0;
                config.free_space = 0;
            }
        }

        Ok(())
    }

    /// 現在のNAS設定を取得
    pub async fn get_configs(&self) -> Vec<NasConfig> {
        self.nas_configs.read().await.clone()
    }

    /// 特定のNASの転送状態を更新
    pub async fn set_transfer_status(&self, nas_id: u32, is_transfer: bool) -> Result<(), String> {
        let mut configs = self.nas_configs.write().await;

        if let Some(config) = configs.iter_mut().find(|c| c.id == nas_id) {
            config.is_transfer = is_transfer;
            Ok(())
        } else {
            Err(format!("NAS with id {} not found", nas_id))
        }
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
