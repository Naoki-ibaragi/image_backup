import { useState } from 'react'
import { Save } from 'lucide-react'

function Settings() {
  const [backupStartTime, setBackupStartTime] = useState('00:00')
  const [imageFolderPath, setImageFolderPath] = useState('')
  const [resultFolderPath, setResultFolderPath] = useState('')

  const handleSave = async () => {
    try {
      console.log('設定を保存:', {
        backupStartTime,
        imageFolderPath,
        resultFolderPath
      })
      // TODO: バックエンドへの保存処理を実装
    } catch (error) {
      console.error('設定の保存に失敗しました:', error)
    }
  }

  return (
    <div className="max-w-3xl mx-auto">
      <div className="bg-white rounded-lg shadow-md p-6 space-y-6">

        {/* バックアップ開始時刻 */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-gray-700">
            バックアップ開始時刻
          </label>
          <input
            type="time"
            value={backupStartTime}
            onChange={(e) => setBackupStartTime(e.target.value)}
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all text-gray-900"
          />
        </div>

        {/* NASフォルダパス-画像 */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-gray-700">
            NASフォルダパス - 画像
          </label>
          <input
            type="text"
            value={imageFolderPath}
            onChange={(e) => setImageFolderPath(e.target.value)}
            placeholder="例: \\192.168.1.100\images"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all text-gray-900 placeholder-gray-400"
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
            placeholder="例: \\192.168.1.100\results"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition-all text-gray-900 placeholder-gray-400"
          />
        </div>

        {/* 保存ボタン */}
        <div className="pt-4">
          <button
            onClick={handleSave}
            className="w-full flex items-center justify-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors shadow-md hover:shadow-lg"
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