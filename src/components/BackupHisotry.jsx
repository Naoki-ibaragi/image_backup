import { useNASContext } from "../contexts/NASContext";

export default function BackupHisotry() {

    const { historyList } = useNASContext(); // グローバルなNAS・外観検査機一覧

    // バイト数を読みやすい形式に変換
    const formatBytes = (bytes) => {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
    };

    // 秒数を読みやすい形式に変換
    const formatDuration = (seconds) => {
        if (seconds < 60) return `${seconds}秒`;
        const minutes = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${minutes}分${secs}秒`;
    };

    // 日時をフォーマット
    const formatDateTime = (dateString) => {
        if (!dateString) return '-';
        const date = new Date(dateString);
        return date.toLocaleString('ja-JP', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        });
    };

    return (
        <div className="space-y-4 p-4">
            {
                [...historyList].reverse().map((history, index) => {
                    return (
                        <div key={index} className="bg-white rounded-lg shadow-md p-6 border-l-4"
                             style={{ borderLeftColor: history.success ? '#10b981' : '#ef4444' }}>

                            {/* ステータス表示 */}
                            <div className="flex items-center justify-between mb-4">
                                <div className="flex items-center gap-3">
                                    <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                                        history.complete
                                            ? 'bg-blue-100 text-blue-800'
                                            : 'bg-red-100 text-red-800'
                                    }`}>
                                        {history.complete ? "実行完了" : "実行失敗"}
                                    </span>
                                    <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                                        history.success
                                            ? 'bg-green-100 text-green-800'
                                            : 'bg-yellow-100 text-yellow-800'
                                    }`}>
                                        {history.success ? "エラーなし" : "エラーあり"}
                                    </span>
                                </div>
                            </div>

                            {/* 日時情報 */}
                            <div className="grid grid-cols-2 gap-4 mb-4 pb-4 border-b border-gray-200">
                                <div>
                                    <p className="text-xs text-gray-500 mb-1">開始時刻</p>
                                    <p className="text-sm font-medium text-gray-700">{formatDateTime(history.start_date)}</p>
                                </div>
                                <div>
                                    <p className="text-xs text-gray-500 mb-1">終了時刻</p>
                                    <p className="text-sm font-medium text-gray-700">{formatDateTime(history.end_date)}</p>
                                </div>
                            </div>

                            {/* 処理情報 */}
                            <div className="grid grid-cols-2 gap-4 mb-4">
                                <div className="bg-gray-50 rounded p-3">
                                    <p className="text-xs text-gray-500 mb-1">総処理ファイル数</p>
                                    <p className="text-lg font-semibold text-gray-800">{history.total_files}</p>
                                </div>
                                <div className="bg-gray-50 rounded p-3">
                                    <p className="text-xs text-gray-500 mb-1">コピー実施ファイル数</p>
                                    <p className="text-lg font-semibold text-gray-800">{history.copied_files}</p>
                                </div>
                                <div className="bg-gray-50 rounded p-3">
                                    <p className="text-xs text-gray-500 mb-1">コピー失敗ファイル数</p>
                                    <p className="text-lg font-semibold text-gray-800">{history.failed_files}</p>
                                </div>
                                <div className="bg-gray-50 rounded p-3">
                                    <p className="text-xs text-gray-500 mb-1">総コピーサイズ</p>
                                    <p className="text-lg font-semibold text-gray-800">{formatBytes(history.total_size_bytes)}</p>
                                </div>
                                <div className="bg-gray-50 rounded p-3">
                                    <p className="text-xs text-gray-500 mb-1">処理時間</p>
                                    <p className="text-lg font-semibold text-gray-800">{formatDuration(history.duration_secs)}</p>
                                </div>
                            </div>

                            {/* エラー内容（エラーがある場合のみ表示） */}
                            {history.errors && history.errors.length > 0 && (
                                <div className="bg-red-50 rounded p-4 border border-red-200">
                                    <p className="text-sm font-medium text-red-800 mb-2">エラー内容:</p>
                                    <div className="space-y-1">
                                        {history.complete ? history.errors.map((error, errorIndex) => (
                                            <p key={errorIndex} className="text-sm text-red-700">
                                                • {error}
                                            </p>
                                        ))
                                        :
                                        <p className="text-sm text-red-700">
                                            • {history.errors}
                                        </p>
                                        }
                                    </div>
                                </div>
                            )}
                        </div>
                    )
                })
            }
        </div>
    );
}
