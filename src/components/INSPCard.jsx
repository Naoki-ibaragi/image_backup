import React, { useState } from "react";
import { ChevronDown, CheckCircle, XCircle, Wifi, Camera, Square, Cog, Trash2 } from "lucide-react";
import { useNASContext } from "../contexts/NASContext";

/**
 * 個別のINSPカードコンポーネント
 * @param {Object} props
 * @param {Object} props.insp - INSP情報
 */
export default function INSPCard({insp}) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const { isBackupRunning } = useNASContext(); // グローバルなNAS・外観検査機一覧
  const isBackup = insp.is_backup === true; //バックアップを実施するかどうか

  return (
    <div className={isExpanded ? "bg-sky-100 rounded-lg mb-2 overflow-hidden shadow-md":"bg-sky-200 rounded-lg mb-2 overflow-hidden shadow-md"}>
      {/* カードヘッダー（常に表示） */}
      <div className="w-full p-4 flex items-center gap-3">
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="flex-1 flex items-center gap-3 hover:bg-gray-750 transition-colors rounded p-2 -m-2"
        >
          <Camera className={isBackup ? "text-blue-600" : "text-gray-500"} size={24} />
          <div className="flex-1 text-left">
            <h3 className="text-lg font-semibold text-black text-mono">{insp.name}</h3>
          </div>
          <span
            className={`flex items-center gap-1 px-3 py-1 rounded-full text-sm font-medium ${
              isBackup
                ? "bg-green-900/20 text-green-700"
                : "bg-red-900/20 text-red-600"
            }`}
          >
            {isBackup ? (
              <>
                <CheckCircle size={16} />
                この検査機器はバックアップ対象です
              </>
            ) : (
              <>
                <XCircle size={16} />
                この検査機器はバックアップしません
              </>
            )}
          </span>
          <ChevronDown
            className={`text-gray-400 transition-transform ${
              isExpanded ? "rotate-180" : ""
            }`}
            size={20}
          />
        </button>

      </div>

      {/* 展開時の詳細情報 */}
      {isExpanded && (
        <div className="px-4 pb-4 border-t ">
          <div className="pt-4 space-y-4">
            <div>
              <p className="text-sm text-gray-400 mb-1">IPアドレス</p>
              <p className="text-black font-mono">{insp.ip}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">表面外観画像フォルダパス</p>
              <p className="text-black font-mono">{insp.surface_image_path}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">裏面外観画像フォルダパス</p>
              <p className="text-black font-mono">{insp.back_image_path}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">Resultファイルフォルダパス</p>
              <p className="text-black font-mono">{insp.result_path}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">最終バックアップ完了時刻</p>
              <p className="text-black">{insp.lastBackuped}</p>
            </div>

            {/* 情報編集ボタン */}
            <div className="border-t border-gray-700 pt-4">
              <button
                disabled={isBackupRunning}
                className={`w-full flex items-center justify-center gap-2 px-4 py-1 rounded-lg transition-colors ${
                  isBackupRunning
                    ? "bg-gray-700 text-gray-500 cursor-not-allowed"
                    : "bg-green-700 hover:bg-green-600 text-white"
                }`}
              >
                <Cog size={16} />
                {isBackupRunning ? "バックアップ処理中は編集できません" : "この検査装置の情報を編集"}
              </button>
            </div>

            {/* バックアップ停止ボタン */}
            <div className="border-t border-gray-700 pt-4">
              <button
                disabled={isBackupRunning}
                className={`w-full flex items-center justify-center gap-2 px-4 py-1 rounded-lg transition-colors ${
                  isBackupRunning
                    ? "bg-gray-700 text-gray-500 cursor-not-allowed"
                    : "bg-orange-700 hover:bg-orange-600 text-white"
                }`}
              >
                <Cog size={16} />
                {isBackupRunning ? "バックアップ処理中は編集できません" : "この検査装置のバックアップを停止"}
              </button>
            </div>

            {/* 削除ボタン */}
            <div className="border-t border-gray-700 pt-4">
              <button
                disabled={isBackupRunning}
                className={`w-full flex items-center justify-center gap-2 px-4 py-1 rounded-lg transition-colors ${
                  isBackupRunning
                    ? "bg-gray-700 text-gray-500 cursor-not-allowed"
                    : "bg-red-700 hover:bg-red-600 text-white"
                }`}
              >
                <Trash2 size={16} />
                {isBackupRunning ? "バックアップ処理中は削除できません" : "この検査装置を削除"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
