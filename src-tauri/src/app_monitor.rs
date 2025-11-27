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
    pub enable_nas_list: Arc<RwLock<Vec<u32>>>,
    pub enable_insp_list: Arc<RwLock<Vec<u32>>>,
}

impl AppMonitor {
    /// 新しいAppMonitorインスタンスを作成
    pub fn new(nas_configs: Vec<NasConfig>, insp_configs: Vec<InspConfig>) -> Self {
        let mut enable_nas_list:Vec<u32>=vec![];
        let mut enable_insp_list:Vec<u32>=vec![];
        //nas_configsから接続可能なnasのリストを取得する
        for nas_config in &nas_configs{
            if nas_config.is_connected==true {
                enable_nas_list.push(nas_config.id);
            }
        }

        //insp_configsから外観機器のリストを取得する
        for insp_config in &insp_configs{
            if insp_config.is_backup==true {
                enable_insp_list.push(insp_config.id);
            }
        }

        Self {
            nas_configs: Arc::new(RwLock::new(nas_configs)),
            insp_configs: Arc::new(RwLock::new(insp_configs)),
            enable_nas_list:Arc::new(RwLock::new(enable_nas_list)),
            enable_insp_list:Arc::new(RwLock::new(enable_insp_list)),
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

                // フロントエンドに更新を通知
                let nas_configs = self.nas_configs.read().await;
                if let Err(e) = app_handle.emit("nas-status-updated", nas_configs.clone()) {
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
pub fn check_nas_connection(nas_ip: &str) -> bool {
    let address = format!("{}:445", nas_ip);

    match TcpStream::connect_timeout(
        &address.parse().unwrap(),
        StdDuration::from_secs(1)
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
    // ドライブレターを正規化（例: "P:" -> "P:\\"）
    let drive_path = if drive_letter.ends_with(":\\") {
        drive_letter.to_string()
    } else if drive_letter.ends_with(":") {
        format!("{}\\", drive_letter)
    } else {
        format!("{}:\\", drive_letter)
    };

    // Windows用の実装
    #[cfg(windows)]
    {
        get_drive_space_info_windows(&drive_path)
    }

    // Windows以外の場合はsysinfoを使用
    #[cfg(not(windows))]
    {
        let disks = Disks::new_with_refreshed_list();

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
}

/// Windows専用: Win32 APIを使用してドライブ容量を取得（ネットワークドライブ対応）
#[cfg(windows)]
fn get_drive_space_info_windows(drive_path: &str) -> Result<DriveSpaceInfo, String> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    // drive_pathを"P:\\"のような形式にする
    let path: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        let result = GetDiskFreeSpaceExW(
            PCWSTR(path.as_ptr()),
            Some(&mut free_bytes_available as *mut u64 as *mut _),
            Some(&mut total_bytes as *mut u64 as *mut _),
            Some(&mut total_free_bytes as *mut u64 as *mut _),
        );

        if result.is_ok() {
            let used = total_bytes.saturating_sub(total_free_bytes);

            println!("Drive {} - Total: {} bytes, Free: {} bytes, Used: {} bytes",
                     drive_path, total_bytes, total_free_bytes, used);

            Ok(DriveSpaceInfo {
                total: total_bytes,
                used,
                free: total_free_bytes,
            })
        } else {
            Err(format!("Failed to get disk space info for drive {}", drive_path))
        }
    }
}
