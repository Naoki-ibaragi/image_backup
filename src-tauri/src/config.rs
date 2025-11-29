use std::fs;
use std::path::PathBuf;
use tauri::command;
use serde_json::{Value, json};
use std::collections::HashMap;

//独自クレートのimport
use crate::types::{NasInfos,InspInfos,NasConfig,InspConfig,Configs,SettingsConfig,InspInfo};
use crate::app_monitor::{check_nas_connection};

/// 設定ファイルの読み込みで初期化
#[command]
pub async fn init_info() -> Result<(Configs,SettingsConfig), String> {
    // 実行ファイルのディレクトリからconfig.jsonを読み込む
    let config_path = get_config_path()?;

    // デバッグ用: パスを出力
    println!("Trying to read config from: {:?}", config_path);

    // ファイルを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // valueで受け取る
    let value:Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    //nas情報を取得
    let nas_info: NasInfos = serde_json::from_value(value["nas_units"].clone())
    .map_err(|e| format!("Failed to parse nas_units: {}", e))?;

    //insp情報を取得
    let insp_info: InspInfos = serde_json::from_value(value["insp_units"].clone())
    .map_err(|e| format!("Failed to parse nas_units: {}", e))?;

    let settings_info: SettingsConfig = serde_json::from_value(value["settings"].clone())
    .map_err(|e| format!("Failed to parse settings: {}", e))?;

    //各NAS情報を追加
    let mut nas_configs = vec![];
    for data in nas_info.nass {
        let nas_config = NasConfig {
            id: data.id,
            name: data.name,
            drive: data.drive,
            nas_ip: data.nas_ip.clone(),
            total_space: 0,
            used_space: 0,
            free_space: 0,
            is_use: true,        //このNASを使用するかどうか(NASに接続できていてもここがfalseだと使用しない)
            is_connected: check_nas_connection(&data.nas_ip), //NASに接続できているか
        };

        nas_configs.push(nas_config);
    }

    //各insp情報を追加
    let mut insp_configs = vec![];
    for data in insp_info.insps {
        let insp_config = InspConfig {
            id: data.id,
            name: data.name,
            insp_ip:data.insp_ip,
            surface_image_path:data.surface_image_path,
            back_image_path:data.back_image_path,
            result_path:data.result_path,
            is_backup:data.is_backup, //バックアップを実施するかどうか(config.jsonから読み込み)
        };

        insp_configs.push(insp_config);
    }

    Ok((Configs{nas_configs:nas_configs,insp_configs:insp_configs},settings_info))
}

/// 設定をconfig.jsonに保存
#[command]
pub async fn save_settings(settings: SettingsConfig) -> Result<(), String> {
    let config_path = get_config_path()?;

    // 既存のconfig.jsonを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // JSONとしてパース
    let mut value: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    // settings部分を更新
    value["settings"] = json!({
        "backup_time": settings.backup_time,
        "surface_image_path": settings.surface_image_path,
        "back_image_path": settings.back_image_path,
        "result_file_path": settings.result_file_path,
    });

    // ファイルに書き込む（インデント付き）
    let updated_content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write config file at {:?}: {}", config_path, e))?;

    println!("Settings saved successfully to {:?}", config_path);
    Ok(())
}

//更新した外観検査の設定をconfig.jsonに保存
#[command]
pub async fn save_insp_settings(insp: InspInfo) -> Result<(), String> {
    let config_path = get_config_path()?;

    // 既存のconfig.jsonを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // JSONとしてパース
    let mut value: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    let mut insp_info: InspInfos = serde_json::from_value(value["insp_units"].clone())
    .map_err(|e| format!("Failed to parse nas_units: {}", e))?;

    //idが一致する情報を更新
    for info in &mut insp_info.insps{
        if info.id==insp.id{
            info.name=insp.name.clone();
            info.insp_ip=insp.insp_ip.clone();
            info.surface_image_path=insp.surface_image_path.clone();
            info.back_image_path=insp.back_image_path.clone();
            info.result_path=insp.result_path.clone();
            info.is_backup=insp.is_backup;
        }
    }

    value["insp_units"]["insps"] = json!(
        insp_info.insps
    );

    // ファイルに書き込む（インデント付き）
    let updated_content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write config file at {:?}: {}", config_path, e))?;

    println!("Settings saved successfully to {:?}", config_path);
    Ok(())
}

/// バックアップ設定の切り替えをconfig.jsonに保存
#[command]
pub async fn save_insp_backup_setting(insp_id: u32, is_backup: bool) -> Result<(), String> {
    let config_path = get_config_path()?;

    // 既存のconfig.jsonを読み込む
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    // JSONとしてパース
    let mut value: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    let mut insp_info: InspInfos = serde_json::from_value(value["insp_units"].clone())
        .map_err(|e| format!("Failed to parse insp_units: {}", e))?;

    // idが一致する情報のis_backupを更新
    for info in &mut insp_info.insps {
        if info.id == insp_id {
            info.is_backup = is_backup;
            break;
        }
    }

    value["insp_units"]["insps"] = json!(insp_info.insps);

    // ファイルに書き込む（インデント付き）
    let updated_content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, updated_content)
        .map_err(|e| format!("Failed to write config file at {:?}: {}", config_path, e))?;

    println!("Backup setting saved successfully to {:?}", config_path);
    Ok(())
}

fn get_config_path()->Result<PathBuf,String>{
    // 開発時とリリース時でパスを変える
    #[cfg(debug_assertions)]
    {
        // 開発時: 現在のディレクトリを確認して適切なパスを構築
        let current = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        println!("Current directory: {:?}", current);

        // src-tauriディレクトリにいる場合は、そのままconfig.jsonを探す
        let mut path = current.clone();
        path.push("config.json");
        if path.exists() {
            return Ok(path);
        }

        // プロジェクトルートにいる場合は、src-tauri/config.jsonを探す
        let mut path = current;
        path.push("src-tauri");
        path.push("config.json");
        Ok(path)
    }

    #[cfg(not(debug_assertions))]
    {
        // リリース時: 実行ファイルと同じディレクトリからconfig.jsonを読む
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;
        let mut config_path = exe_path.parent()
            .ok_or("Failed to get parent directory")?
            .to_path_buf();
        config_path.push("config.json");
        Ok(config_path)
    }

}
