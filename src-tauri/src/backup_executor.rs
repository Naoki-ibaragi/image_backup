use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};
use crate::types::{InspConfig, NasConfig, SettingsConfig, BackupResult, BackupProgress};
use std::collections::HashMap;
use walkdir::WalkDir;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_SECS: u64 = 5;

/// バックアップ実行を担当する構造体
pub struct BackupExecutor;

impl BackupExecutor {
    /// バックアップを実行
    pub async fn execute(
        insp_configs: Vec<InspConfig>,
        nas_configs: Vec<NasConfig>,
        settings: SettingsConfig,
        app_handle: AppHandle,
        last_backup_nas_id:Option<u32>
    ) -> Result<BackupResult, String> {
        let start_time = Instant::now();
        let mut total_files = 0u64;
        let mut copied_files = 0u64;
        let mut failed_files = 0u64;
        let mut total_size = 0u64;
        let mut errors = Vec::new();

        log::info!("Starting backup process...");

        // バックアップ対象の検査機器のみをフィルタ
        let active_insp_configs: Vec<&InspConfig> = insp_configs
            .iter()
            .filter(|insp| insp.is_backup)
            .collect();

        // 使用可能で接続されているNASのみをフィルタ
        let mut active_nas_configs: Vec<&NasConfig> = nas_configs
            .iter()
            .filter(|nas| nas.is_use && nas.is_connected)
            .collect();

        if active_nas_configs.is_empty() {
            return Err("利用可能なNASがありません".to_string());
        }

        if active_insp_configs.is_empty() {
            return Err("バックアップ対象の検査機器がありません".to_string());
        }

        //last_backuped_nas情報から今回バックアップをリトライするNASの順序を確定する
        let mut active_nas_ids: Vec<u32> = active_nas_configs
            .iter()
            .map(|nas| nas.id)
            .collect();

        active_nas_ids.sort();
        let rotation_nas_ids = match last_backup_nas_id {
            None => {
                active_nas_ids
            }
            Some(v) => {
                //rotation_nas_idsを作成
                //ex:nas_id_list=[1,2,3,4] last_backup_nas_id=2 => rotation_nas_list=[2,3,4,1]とする
                match Self::rotate_to_value(&active_nas_ids, v) {
                    Some(vec) => vec,
                    None => { //前回の最終nasidが無くなっていたときは若いidから再度始める
                        active_nas_ids
                    }
                }
            }
        };

        // active_nas_configsをrotation_nas_idsの順番で並び替え
        active_nas_configs.sort_by_key(|nas| {
            rotation_nas_ids
                .iter()
                .position(|&id| id == nas.id)
                .unwrap_or(usize::MAX)
        });

        log::info!("Active inspection devices: {}", active_insp_configs.len());
        log::info!("Active NAS devices: {}", active_nas_configs.len());

        // 現在使用中のNASインデックス (容量不足時に切り替え)
        let mut nas_index = 0usize;

        // 各検査機器からバックアップを実行
        for insp_config in active_insp_configs {
            log::info!("Processing device: {}", insp_config.name);

            // すべてのNASから既存データを収集（重複チェック用）
            let nas_surface_image_map = Self::collect_all_nas_folder_data(
                &active_nas_configs,
                &settings.surface_image_path,
                &insp_config.name,
            );
            let nas_back_image_map = Self::collect_all_nas_folder_data(
                &active_nas_configs,
                &settings.back_image_path,
                &insp_config.name,
            );
            let nas_result_file_map = Self::collect_all_nas_folder_data(
                &active_nas_configs,
                &settings.result_file_path,
                &insp_config.name,
            );

            // バックアップ処理（NAS容量チェック付き）
            loop {
                // NAS容量チェック
                if nas_index >= active_nas_configs.len() {
                    return Err(format!(
                        "すべてのNASで容量不足です。バックアップを中断しました。(検査機器: {})",
                        insp_config.name
                    ));
                }

                let nas_config = active_nas_configs[nas_index];
                log::info!("  Backing up to NAS: {} (Index: {})", nas_config.name, nas_index);

                // NASの空き容量をチェック
                if nas_config.free_space < settings.required_free_space {
                    log::warn!(
                        "  NAS {} の空き容量不足: {} < {} (required). 次のNASに切り替えます...",
                        nas_config.name,
                        nas_config.free_space,
                        settings.required_free_space
                    );
                    nas_index += 1;
                    continue; // 次のNASへ
                }

                // 表面画像のバックアップ（差分のみ）
                if !insp_config.surface_image_path.is_empty() {
                    match Self::backup_folder_with_diff(
                        &insp_config.insp_ip,
                        &insp_config.surface_image_path,
                        &nas_config.drive,
                        &settings.surface_image_path,
                        &insp_config.name,
                        "表面画像",
                        &nas_surface_image_map,
                        &app_handle,
                    ).await {
                        Ok(stats) => {
                            total_files += stats.0;
                            copied_files += stats.1;
                            failed_files += stats.2;
                            total_size += stats.3;
                        }
                        Err(e) => {
                            errors.push(format!("{} - 表面画像: {}", insp_config.name, e));
                        }
                    }
                }

                // 裏面画像のバックアップ（差分のみ）
                if !insp_config.back_image_path.is_empty() {
                    match Self::backup_folder_with_diff(
                        &insp_config.insp_ip,
                        &insp_config.back_image_path,
                        &nas_config.drive,
                        &settings.back_image_path,
                        &insp_config.name,
                        "裏面画像",
                        &nas_back_image_map,
                        &app_handle,
                    ).await {
                        Ok(stats) => {
                            total_files += stats.0;
                            copied_files += stats.1;
                            failed_files += stats.2;
                            total_size += stats.3;
                        }
                        Err(e) => {
                            errors.push(format!("{} - 裏面画像: {}", insp_config.name, e));
                        }
                    }
                }

                // 結果ファイルのバックアップ（差分のみ）
                if !insp_config.result_path.is_empty() {
                    match Self::backup_folder_with_diff(
                        &insp_config.insp_ip,
                        &insp_config.result_path,
                        &nas_config.drive,
                        &settings.result_file_path,
                        &insp_config.name,
                        "結果ファイル",
                        &nas_result_file_map,
                        &app_handle,
                    ).await {
                        Ok(stats) => {
                            total_files += stats.0;
                            copied_files += stats.1;
                            failed_files += stats.2;
                            total_size += stats.3;
                        }
                        Err(e) => {
                            errors.push(format!("{} - 結果ファイル: {}", insp_config.name, e));
                        }
                    }
                }

                // このNASへのバックアップ成功、次の検査機器へ
                log::info!("  NAS {} へのバックアップ完了", nas_config.name);
                break;
            }
        }

        let duration = start_time.elapsed().as_secs();
        let success = failed_files == 0 && errors.is_empty();

        log::info!("Backup completed: {} files copied, {} failed", copied_files, failed_files);

        Ok(BackupResult {
            success,
            total_files,
            copied_files,
            failed_files,
            total_size_bytes: total_size,
            duration_secs: duration,
            errors,
        })
    }

    /// コピー先のパスを構築
    fn build_dest_path(drive: &str, base_path: &str, device_name: &str) -> String {
        let drive_clean = drive.trim_end_matches(":\\").trim_end_matches(":");
        format!("{}:\\{}\\{}", drive_clean, base_path.trim_start_matches("\\"), device_name)
    }

    ///ベクトルを任意の値から開始する
    fn rotate_to_value(vec: &[u32], value: u32) -> Option<Vec<u32>> {
    // 指定した値のインデックスを探す
        let pos = vec.iter().position(|&x| x == value)?;

        // 新しい Vec を作成
        let mut new_vec = Vec::with_capacity(vec.len());

        // pos から最後まで
        new_vec.extend_from_slice(&vec[pos..]);
        // 先頭から pos まで
        new_vec.extend_from_slice(&vec[..pos]);

        Some(new_vec)
    }

    /// すべてのNASから既存フォルダデータを収集する（重複チェック用）
    fn collect_all_nas_folder_data(
        nas_configs: &[&NasConfig],
        base_path: &str,
        device_name: &str,
    ) -> HashMap<String, u32> {
        let mut all_file_map = HashMap::new();

        for nas_config in nas_configs {
            let dest_path = Self::build_dest_path(
                &nas_config.drive,
                base_path,
                device_name,
            );
            Self::get_all_file_data(dest_path, &mut all_file_map);
        }

        all_file_map
    }

    ///フォルダ内の全フォルダに対してpath_nameとフォルダ内のファイル数のhashmapを作成する
    fn get_all_file_data(folder_path:String,all_file_map: &mut HashMap<String,u32>){
        //dest_path内の各ロット番号フォルダ一覧に対してフォルダ内のファイル数をhashmapに保存する 
        let entry_list=match fs::read_dir(folder_path){
            Ok(v)=>v,
            _=>return
        };

        for entry in entry_list {
            let entry=match entry{
                Ok(v)=>v,
                _=>continue
            };
            let metadata = match entry.metadata(){
                Ok(v)=>v,
                _=>continue
            };

            //NASの各ロット名フォルダ内のファイル数を再帰的に取得する 
            if metadata.is_dir() {
                let mut count=0;
                for entry in WalkDir::new(entry.path()).into_iter().filter_map(|e| e.ok()) {
                    if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
                        count += 1;
                    }
                }

                if let Some(p)=entry.path().file_name(){
                    all_file_map.entry(p.to_string_lossy().to_string()).or_insert(count);
                }

            }
        }

    }

    /// 差分バックアップを実行（既にNASにあるフォルダはスキップ）
    async fn backup_folder_with_diff(
        insp_ip: &str,
        source_relative_path: &str,
        nas_drive: &str,
        nas_base_path: &str,
        device_name: &str,
        category: &str,
        existing_folders: &HashMap<String, u32>,
        app_handle: &AppHandle,
    ) -> Result<(u64, u64, u64, u64), String> {
        // 検査機器側のソースパスを構築
        let mut source_path = PathBuf::new();
        source_path.push(format!("\\\\{}", insp_ip)); // UNCパス形式
        source_path.push(source_relative_path);

        // NAS側のコピー先パスを取得
        let dest_path = Self::build_dest_path(nas_drive, nas_base_path, device_name);

        log::debug!("ソースパス: {}", source_path.display());
        log::debug!("コピー先パス: {}", dest_path);

        let mut total_files = 0u64;
        let mut copied_files = 0u64;
        let mut failed_files = 0u64;
        let mut total_size = 0u64;

        // ソースフォルダ内のエントリを読み込み
        let entries = match fs::read_dir(&source_path) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(format!(
                    "ソースパス読み込みエラー {}: {}",
                    source_path.display(),
                    e
                ));
            }
        };

        // フォルダ単位でコピーするかチェックしながら進める
        for entry_result in entries {
            let entry = match entry_result {
                Ok(v) => v,
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(v) => v,
                Err(_) => continue,
            };

            // ファイルはスキップ（フォルダのみ処理）
            if metadata.is_file() {
                continue;
            }

            // 既にNASにあるデータかチェック（ファイル数が一致すればスキップ）
            if !Self::should_copy_folder(&entry, existing_folders) {
                log::debug!(
                    "    スキップ: {} (既にNASに存在)",
                    entry.file_name().to_string_lossy()
                );
                continue;
            }

            // entry単位でコピーする
            match Self::copy_with_retry(&entry, &dest_path, device_name, category, app_handle).await
            {
                Ok(stats) => {
                    total_files += stats.0;
                    copied_files += stats.1;
                    failed_files += stats.2;
                    total_size += stats.3;
                }
                Err(e) => {
                    failed_files += 1;
                    log::error!(
                        "フォルダコピー失敗: {} - {}",
                        entry.file_name().to_string_lossy(),
                        e
                    );
                }
            }
        }

        Ok((total_files, copied_files, failed_files, total_size))
    }

    /// フォルダをコピーすべきかチェック
    /// 戻り値: true = コピーする, false = スキップする
    fn should_copy_folder(entry: &DirEntry, nas_data_hashmap: &HashMap<String, u32>) -> bool {
        //検査機器側のコピー元ロット番号名フォルダ
        let folder_name = entry.file_name().to_string_lossy().to_string();

        // entry内のファイル数を取得
        let file_count = WalkDir::new(entry.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
            .count() as u32;

        // NASに既に存在するかチェック
        match nas_data_hashmap.get(&folder_name) {
            Some(&nas_file_count) => {
                // ファイル数が一致する場合はスキップ（既にバックアップ済み）
                if nas_file_count == file_count {
                    false // スキップ
                } else {
                    // ファイル数が違う場合はコピー（更新が必要）
                    log::info!(
                        "    ファイル数不一致: {} (NAS: {}, 検査機器: {})",
                        folder_name, nas_file_count, file_count
                    );
                    true // コピーする
                }
            }
            None => {
                // NASに存在しない場合はコピー
                true // コピーする
            }
        }
    }
    
    /// リトライ付きでディレクトリをコピー
    async fn copy_with_retry(
        entry:&DirEntry,
        dest: &str,
        device_name: &str,
        category: &str,
        app_handle: &AppHandle,
    ) -> Result<(u64, u64, u64, u64), String> {
        let mut last_error = String::new();

        for attempt in 1..=MAX_RETRIES {
            match Self::copy_directory(entry, dest,device_name, category, app_handle).await {
                Ok(result) => {
                    if attempt > 1 {
                        log::info!("  リトライ成功 (試行 {}/{}): {} - {}", attempt, MAX_RETRIES, device_name, category);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = e.clone();
                    if attempt < MAX_RETRIES {
                        log::warn!("  コピー失敗 (試行 {}/{}): {} - {} - エラー: {}",
                            attempt, MAX_RETRIES, device_name, category, e);
                        log::info!("  {}秒後にリトライします...", RETRY_DELAY_SECS);
                        sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    } else {
                        log::error!("  コピー失敗 (最終試行): {} - {} - エラー: {}",
                            device_name, category, e);
                    }
                }
            }
        }

        Err(format!("{}回のリトライ後も失敗: {}", MAX_RETRIES, last_error))
    }

    /// ディレクトリを再帰的にコピー
    /// 戻り値: (total_files, copied_files, failed_files, total_size)
    async fn copy_directory(
        entry:&DirEntry,
        dest: &str,
        device_name: &str,
        category: &str,
        app_handle: &AppHandle,
    ) -> Result<(u64, u64, u64, u64), String> {
        let mut dest_path = PathBuf::new();     //NAS側のパス
        dest_path.push(dest);
        dest_path.push(entry.file_name().to_string_lossy().to_string());

        if !entry.path().exists() {
            return Err(format!("検査機器側のコピー元フォルダが存在しません: {}", entry.path().to_string_lossy().to_string()));
        }

        // コピー先ディレクトリを作成
        fs::create_dir_all(&dest_path)
            .map_err(|e| format!("ディレクトリ作成エラー {}: {}", dest_path.display(), e))?;

        let mut total_files = 0u64;
        let mut copied_files = 0u64;
        let mut failed_files = 0u64;
        let mut total_size = 0u64;

        // ディレクトリ内のファイルをコピー
        Self::copy_files_recursive(
            entry.path().as_path(),
            dest_path.as_path(),
            device_name,
            category,
            app_handle,
            &mut total_files,
            &mut copied_files,
            &mut failed_files,
            &mut total_size,
        )?;

        Ok((total_files, copied_files, failed_files, total_size))
    }

    /// ディレクトリを再帰的にコピー（内部実装）
    fn copy_files_recursive(
        source: &Path,
        dest: &Path,
        device_name: &str,
        category: &str,
        app_handle: &AppHandle,
        total_files: &mut u64,
        copied_files: &mut u64,
        failed_files: &mut u64,
        total_size: &mut u64,
    ) -> Result<(), String> {
        if source.is_dir() {
            // ディレクトリの場合、中身を再帰的にコピー
            let entries = fs::read_dir(source)
                .map_err(|e| format!("ディレクトリ読み込みエラー {}: {}", source.display(), e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("エントリ読み込みエラー: {}", e))?;
                let source_path = entry.path();
                let file_name = entry.file_name();
                let dest_path = dest.join(&file_name);

                if source_path.is_dir() {
                    //ロット番号名フォルダの中にさらにフォルダがある場合、それをNASにも作成
                    fs::create_dir_all(&dest_path)
                        .map_err(|e| format!("ディレクトリ作成エラー {}: {}", dest_path.display(), e))?;

                    Self::copy_files_recursive(
                        &source_path,
                        &dest_path,
                        device_name,
                        category,
                        app_handle,
                        total_files,
                        copied_files,
                        failed_files,
                        total_size,
                    )?;
                } else {
                    *total_files += 1;

                    // ファイルコピー
                    match fs::copy(&source_path, &dest_path) {
                        Ok(size) => {
                            *copied_files += 1;
                            *total_size += size;

                            // 進捗を通知
                            let progress = BackupProgress {
                                current_files: *copied_files,
                                total_files: *total_files,
                                current_size: *total_size,
                                total_size: *total_size,
                                percentage: (*copied_files as f32 / *total_files as f32) * 100.0,
                                current_file: file_name.to_string_lossy().to_string(),
                                current_device: format!("{} - {}", device_name, category),
                            };

                            let _ = app_handle.emit("backup-progress", progress);
                        }
                        Err(e) => {
                            *failed_files += 1;
                            log::error!("ファイルコピー失敗 {} -> {}: {}",
                                source_path.display(), dest_path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
