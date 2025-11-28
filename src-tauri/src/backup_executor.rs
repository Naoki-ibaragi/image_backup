use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};
use crate::types::{InspConfig, NasConfig, SettingsConfig, BackupResult, BackupProgress};

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
    ) -> Result<BackupResult, String> {
        let start_time = Instant::now();
        let mut total_files = 0u64;
        let mut copied_files = 0u64;
        let mut failed_files = 0u64;
        let mut total_size = 0u64;
        let mut errors = Vec::new();

        println!("Starting backup process...");

        // バックアップ対象の検査機器のみをフィルタ
        let active_insp_configs: Vec<&InspConfig> = insp_configs
            .iter()
            .filter(|insp| insp.is_backup)
            .collect();

        // 使用可能で接続されているNASのみをフィルタ
        let active_nas_configs: Vec<&NasConfig> = nas_configs
            .iter()
            .filter(|nas| nas.is_use && nas.is_connected)
            .collect();

        if active_nas_configs.is_empty() {
            return Err("利用可能なNASがありません".to_string());
        }

        if active_insp_configs.is_empty() {
            return Err("バックアップ対象の検査機器がありません".to_string());
        }

        println!("Active inspection devices: {}", active_insp_configs.len());
        println!("Active NAS devices: {}", active_nas_configs.len());

        // 各検査機器からバックアップを実行
        for insp_config in active_insp_configs {
            println!("Processing device: {}", insp_config.name);

            // 各NASへバックアップ（冗長化のため）
            for nas_config in &active_nas_configs {
                println!("  Backing up to NAS: {}", nas_config.name);

                // 表面画像のバックアップ（リトライ付き）
                if !insp_config.surface_image_path.is_empty() {
                    let dest_path = Self::build_dest_path(
                        &nas_config.drive,
                        &settings.surface_image_path,
                        &insp_config.name,
                    );

                    match Self::copy_with_retry(
                        &insp_config.surface_image_path,
                        &dest_path,
                        &insp_config.name,
                        "表面画像",
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

                // 裏面画像のバックアップ（リトライ付き）
                if !insp_config.back_image_path.is_empty() {
                    let dest_path = Self::build_dest_path(
                        &nas_config.drive,
                        &settings.back_image_path,
                        &insp_config.name,
                    );

                    match Self::copy_with_retry(
                        &insp_config.back_image_path,
                        &dest_path,
                        &insp_config.name,
                        "裏面画像",
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

                // 結果ファイルのバックアップ（リトライ付き）
                if !insp_config.result_path.is_empty() {
                    let dest_path = Self::build_dest_path(
                        &nas_config.drive,
                        &settings.result_file_path,
                        &insp_config.name,
                    );

                    match Self::copy_with_retry(
                        &insp_config.result_path,
                        &dest_path,
                        &insp_config.name,
                        "結果ファイル",
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
            }
        }

        let duration = start_time.elapsed().as_secs();
        let success = failed_files == 0 && errors.is_empty();

        println!("Backup completed: {} files copied, {} failed", copied_files, failed_files);

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

    /// リトライ付きでディレクトリをコピー
    async fn copy_with_retry(
        source: &str,
        dest: &str,
        device_name: &str,
        category: &str,
        app_handle: &AppHandle,
    ) -> Result<(u64, u64, u64, u64), String> {
        let mut last_error = String::new();

        for attempt in 1..=MAX_RETRIES {
            match Self::copy_directory(source, dest, device_name, category, app_handle).await {
                Ok(result) => {
                    if attempt > 1 {
                        println!("  リトライ成功 (試行 {}/{}): {} - {}", attempt, MAX_RETRIES, device_name, category);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = e.clone();
                    if attempt < MAX_RETRIES {
                        println!("  コピー失敗 (試行 {}/{}): {} - {} - エラー: {}",
                            attempt, MAX_RETRIES, device_name, category, e);
                        println!("  {}秒後にリトライします...", RETRY_DELAY_SECS);
                        sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    } else {
                        println!("  コピー失敗 (最終試行): {} - {} - エラー: {}",
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
        source: &str,
        dest: &str,
        device_name: &str,
        category: &str,
        app_handle: &AppHandle,
    ) -> Result<(u64, u64, u64, u64), String> {
        let source_path = Path::new(source);
        let dest_path = Path::new(dest);

        if !source_path.exists() {
            return Err(format!("ソースパスが存在しません: {}", source));
        }

        // コピー先ディレクトリを作成
        fs::create_dir_all(dest_path)
            .map_err(|e| format!("ディレクトリ作成エラー {}: {}", dest, e))?;

        let mut total_files = 0u64;
        let mut copied_files = 0u64;
        let mut failed_files = 0u64;
        let mut total_size = 0u64;

        // ディレクトリ内のファイルをコピー
        Self::copy_dir_recursive(
            source_path,
            dest_path,
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
    fn copy_dir_recursive(
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
                    fs::create_dir_all(&dest_path)
                        .map_err(|e| format!("ディレクトリ作成エラー {}: {}", dest_path.display(), e))?;

                    Self::copy_dir_recursive(
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
                            eprintln!("ファイルコピー失敗 {} -> {}: {}",
                                source_path.display(), dest_path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
