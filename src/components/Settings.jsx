import { useEffect } from 'react'
import { Save } from 'lucide-react'
import { useNASContext } from "../contexts/NASContext";
import { invoke } from '@tauri-apps/api/core';

function Settings() {

  const {
    backupStartTime,
    setBackupStartTime,
    surfaceImageFolderPath,
    setSurfaceImageFolderPath,
    backImageFolderPath,
    setBackImageFolderPath,
    resultFolderPath,
    setResultFolderPath,
    isBackupRunning
  } = useNASContext(); // グローバルなNAS・外観検査機一覧

  // コンポーネントマウント時に設定を読み込む
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const settings = await invoke('get_settings')
        setBackupStartTime(settings.backup_time)
        setSurfaceImageFolderPath(settings.surface_image_path)
        setBackImageFolderPath(settings.back_image_path)
        setResultFolderPath(settings.result_file_path)
      } catch (error) {
        console.error('設定の読み込みに失敗しました:', error)
      }
    }

    loadSettings()
  }, [])

  const handleSave = async () => {
    try {
      const settingsToSave = {
        backup_time: backupStartTime,
        surface_image_path: surfaceImageFolderPath,
        back_image_path: backImageFolderPath,
        result_file_path: resultFolderPath
      }

      await invoke('update_settings', { newSettings: settingsToSave })

      console.log('設定を保存しました:', settingsToSave)
      alert('設定を保存しました')
    } catch (error) {
      console.error('設定の保存に失敗しました:', error)
      alert(`設定の保存に失敗しました: ${error}`)
    }
  }

  return (
    <div className="max-w-3xl mx-auto">
      <div className="bg-white rounded-lg shadow-md p-6 space-y-6">

        {/* バックアップ実行中の警告 */}
        {isBackupRunning && (
          <div className="bg-yellow-100 border-l-4 border-yellow-500 text-yellow-700 p-4 rounded">
            <p className="font-bold">バックアップ実行中</p>
            <p className="text-sm">バックアップが完了するまで設定を変更できません。</p>
          </div>
        )}

        {/* バックアップ開始時刻 */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-gray-700">
            バックアップ開始時刻
          </label>
          <input
            type="time"
            value={backupStartTime}
            onChange={(e) => setBackupStartTime(e.target.value)}
            disabled={isBackupRunning}
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all text-gray-900 disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
        </div>

        {/* NASフォルダパス-表面画像 */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-gray-700">
            NASフォルダパス - 表面画像
          </label>
          <input
            type="text"
            value={surfaceImageFolderPath}
            onChange={(e) => setSurfaceImageFolderPath(e.target.value)}
            disabled={isBackupRunning}
            placeholder="例: \\192.168.1.100\images"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all text-gray-900 placeholder-gray-400 disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
        </div>

        {/* NASフォルダパス-裏面画像 */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-gray-700">
            NASフォルダパス - 裏面画像
          </label>
          <input
            type="text"
            value={backImageFolderPath}
            onChange={(e) => setBackImageFolderPath(e.target.value)}
            disabled={isBackupRunning}
            placeholder="例: \\192.168.1.100\images"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all text-gray-900 placeholder-gray-400 disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
        </div>


        {/* NASフォルダパス-Resultファイル */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-gray-700">
            NASフォルダパス - Resultファイル
          </label>
          <input
            type="text"
            value={resultFolderPath}
            onChange={(e) => setResultFolderPath(e.target.value)}
            disabled={isBackupRunning}
            placeholder="例: \\192.168.1.100\results"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all text-gray-900 placeholder-gray-400 disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
        </div>

        {/* 保存ボタン */}
        <div className="pt-4">
          <button
            onClick={handleSave}
            disabled={isBackupRunning}
            className="w-full flex items-center justify-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors shadow-md hover:shadow-lg disabled:bg-gray-400 disabled:cursor-not-allowed disabled:hover:bg-gray-400 disabled:shadow-none"
          >
            <Save size={20} />
            設定を保存
          </button>
        </div>

      </div>
    </div>
  )
}

export default Settings
