import { useNASContext } from "../contexts/NASContext";

function BackupProgress() {
  const { isBackupRunning, backupProgress } = useNASContext();

  if (!isBackupRunning && !backupProgress) {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 w-96 bg-white rounded-lg shadow-2xl border border-gray-200 p-6 z-50">
      <div className="space-y-4">
        {/* ヘッダー */}
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900">
            バックアップ実行中
          </h3>
          <div className="flex items-center space-x-2">
            <div className="animate-spin h-5 w-5 border-2 border-blue-500 border-t-transparent rounded-full"></div>
            <span className="text-sm text-gray-600">実行中...</span>
          </div>
        </div>

        {/* 進捗情報 */}
        {backupProgress && (
          <>
            {/* プログレスバー */}
            <div className="space-y-2">
              <div className="flex justify-between text-sm text-gray-600">
                <span>進捗状況</span>
                <span className="font-semibold">{backupProgress.percentage.toFixed(1)}%</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-3 overflow-hidden">
                <div
                  className="bg-blue-600 h-full rounded-full transition-all duration-300 ease-out"
                  style={{ width: `${backupProgress.percentage}%` }}
                ></div>
              </div>
            </div>

            {/* 詳細情報 */}
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">処理ファイル数</span>
                <span className="font-semibold text-gray-900">
                  {backupProgress.current_files} / {backupProgress.total_files}
                </span>
              </div>

              <div className="flex justify-between">
                <span className="text-gray-600">データサイズ</span>
                <span className="font-semibold text-gray-900">
                  {formatBytes(backupProgress.current_size)}
                </span>
              </div>

              <div className="flex justify-between">
                <span className="text-gray-600">現在の対象</span>
                <span className="font-semibold text-gray-900 truncate max-w-[200px]" title={backupProgress.current_device}>
                  {backupProgress.current_device}
                </span>
              </div>

              <div className="pt-2 border-t border-gray-200">
                <p className="text-gray-600 text-xs truncate" title={backupProgress.current_file}>
                  ファイル: {backupProgress.current_file}
                </p>
              </div>
            </div>
          </>
        )}

        {/* 状態がない場合 */}
        {!backupProgress && (
          <div className="text-center text-gray-600">
            <p>バックアップを準備しています...</p>
          </div>
        )}
      </div>
    </div>
  );
}

// バイト数を人間が読みやすい形式に変換
function formatBytes(bytes) {
  if (bytes === 0) return '0 B';

  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

export default BackupProgress;
