use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::SettingsConfig;

/// アプリケーションの設定を管理する構造体
#[derive(Clone)]
pub struct SettingsMonitor {
    settings: Arc<RwLock<SettingsConfig>>,
}

impl SettingsMonitor {
    /// 新しいSettingsMonitorインスタンスを作成
    pub fn new(settings_config: SettingsConfig) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings_config)),
        }
    }

    /// 現在の設定を取得
    pub async fn get_settings(&self) -> SettingsConfig {
        self.settings.read().await.clone()
    }

    /// 設定を更新
    pub async fn update_settings(&self, new_settings: SettingsConfig) {
        let mut settings = self.settings.write().await;
        *settings = new_settings;
    }

    /// 開始時刻を取得
    pub async fn get_backup_time(&self) -> String {
        self.settings.read().await.backup_time.clone()
    }

    /// 開始時刻を更新
    pub async fn set_backup_time(&self, backup_time: String) {
        let mut settings = self.settings.write().await;
        settings.backup_time = backup_time;
    }

    /// 表面画像パスを取得
    pub async fn get_surface_image_path(&self) -> String {
        self.settings.read().await.surface_image_path.clone()
    }

    /// 表面画像パスを更新
    pub async fn set_surface_image_path(&self, path: String) {
        let mut settings = self.settings.write().await;
        settings.surface_image_path = path;
    }

    /// 裏面画像パスを取得
    pub async fn get_back_image_path(&self) -> String {
        self.settings.read().await.back_image_path.clone()
    }

    /// 裏面画像パスを更新
    pub async fn set_back_image_path(&self, path: String) {
        let mut settings = self.settings.write().await;
        settings.back_image_path = path;
    }

    /// 結果ファイルパスを取得
    pub async fn get_result_file_path(&self) -> String {
        self.settings.read().await.result_file_path.clone()
    }

    /// 結果ファイルパスを更新
    pub async fn set_result_file_path(&self, path: String) {
        let mut settings = self.settings.write().await;
        settings.result_file_path = path;
    }
}
