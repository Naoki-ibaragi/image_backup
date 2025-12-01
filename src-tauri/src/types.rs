use serde::{Deserialize,Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/*jsonファイル読み込み用 */
/// NAS設定基本情報
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NasInfo {
    pub id: u32,
    pub name: String,
    pub drive: String,
    pub nas_ip: String,
}

/// 外観設定基本情報
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InspInfo {
    pub id: u32,
    pub name: String,
    pub insp_ip: String,
    pub surface_image_path: String,
    pub back_image_path: String,
    pub result_path: String,
    #[serde(default = "default_is_backup")]
    pub is_backup: bool,
}

// デフォルト値としてtrueを返す関数
fn default_is_backup() -> bool {
    true
}

/// Nas基本情報
#[derive(Serialize, Deserialize, Debug)]
pub struct NasInfos {
    pub nass: Vec<NasInfo>,
}

/// Insp基本情報
#[derive(Serialize, Deserialize, Debug)]
pub struct InspInfos {
    pub insps: Vec<InspInfo>,
}
/* ----------------------------- */

/* 実際に使用する構造体 */
/// 設定ファイル全体の構造
#[derive(Serialize, Deserialize, Debug)]
pub struct Configs {
    pub nas_configs: Vec<NasConfig>,
    pub insp_configs: Vec<InspConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NasConfig {
    pub id: u32,
    pub name: String,
    pub drive: String,
    pub nas_ip: String,
    pub is_use: bool,
    pub is_connected: bool,
    pub total_space: u64,
    pub used_space: u64,
    pub free_space: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InspConfig {
    pub id: u32,
    pub name: String,
    pub insp_ip: String,
    pub surface_image_path: String,     //
    pub back_image_path: String,     //
    pub result_path: String,
    pub is_backup: bool,
}

/* ----------------------------- */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsConfig{
    pub backup_time:String,
    pub surface_image_path:String,
    pub back_image_path:String,
    pub result_file_path:String,
    pub required_free_space:u64
}

/* バックアップ関連の型定義 */
/// バックアップの状態
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupStatus {
    pub is_running: bool,
    pub last_backup_date: Option<String>,
}

/// バックアップの進捗情報
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupProgress {
    pub current_files: u64,
    pub total_files: u64,
    pub current_size: u64,
    pub total_size: u64,
    pub percentage: f32,
    pub current_file: String,
    pub current_device: String,
}

/// バックアップの結果
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupResult {
    pub success: bool,
    pub total_files: u64,
    pub copied_files: u64,
    pub failed_files: u64,
    pub total_size_bytes: u64,
    pub duration_secs: u64,
    pub errors: Vec<String>,
}