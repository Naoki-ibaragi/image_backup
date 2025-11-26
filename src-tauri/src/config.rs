use std::fs;
use std::path::PathBuf;
use tauri::command;
use crate::types::{NasInfos,InspInfos,NasConfig,InspConfig};
use serde_json::Value;

/// config.jsonを読み込んでPLC設定情報をフロントエンドに渡す
#[command]
pub async fn init_initial_info() -> Result<(Vec<NasConfig>,Vec<InspConfig>), String> {
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

    //各NAS情報を追加
    let mut nas_configs = vec![];
    for data in nas_info.nass {
        let nas_config = NasConfig {
            id: data.id,
            name: data.name,
            drive: data.drive,
            nas_ip: data.nas_ip,
            is_transfer: false,
            is_connected: false,
            total_space: 0,
            current_space: 0,
            free_space: 0
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
            is_backup:true
        };

        insp_configs.push(insp_config);
    }

    println!("{:#?}",insp_configs);

    Ok((nas_configs,insp_configs))
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