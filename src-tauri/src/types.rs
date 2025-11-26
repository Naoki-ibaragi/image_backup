use serde::{Deserialize,Serialize};

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
    pub surface_image_path: String,     //
    pub back_image_path: String,     //
    pub result_path: String,
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


/// 設定ファイル全体の構造
#[derive(Serialize, Deserialize, Debug)]
pub struct NasConfigs {
    pub nass: Vec<NasConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NasConfig {
    pub id: u32,
    pub name: String,
    pub drive: String,
    pub nas_ip: String,
    pub is_connected: bool,
    pub is_transfer: bool,
    pub total_space: u64,
    pub current_space: u64,
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

