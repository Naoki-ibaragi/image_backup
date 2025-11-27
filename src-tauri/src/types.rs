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

/* nas、外観検査の転送状況を表示 */
pub struct TransferState{
    pub nas_id_current:Arc<RwLock<Option<u32>>>, //現在使用中のnas
    pub insp_id_current:Arc<RwLock<Option<u32>>>, //次に使用予定のnas
    pub nas_id_next:Arc<RwLock<Option<u32>>>, //現在バックアップ中の外観検査
    pub insp_id_next:Arc<RwLock<Option<u32>>>, //次にバックアップ予定の外観検査
}

impl TransferState{
    pub fn new(a:Option<u32>,b:Option<u32>,c:Option<u32>,d:Option<u32>)->Self{
        Self { 
            nas_id_current:Arc::new(RwLock::new(a)), 
            nas_id_next:Arc::new(RwLock::new(b)), 
            insp_id_current:Arc::new(RwLock::new(c)), 
            insp_id_next:Arc::new(RwLock::new(d)), 
        }
    }
}
