import React, { useState } from "react";
import { ChevronDown, CheckCircle, XCircle, Wifi, Camera, Square, Cog, Trash2 } from "lucide-react";
import { useNASContext } from "../contexts/NASContext";
import EditInspDialog from "./EditInspDialog";
import { invoke } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";

/**
 * 個別のINSPカードコンポーネント
 * @param {Object} props
 * @param {Object} props.insp - INSP情報
 */
export default function INSPCard({insp}) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const { isBackupRunning,inspList,setInspList } = useNASContext(); // グローバルなNAS・外観検査機一覧
  const [isEditDialogOpen,setIsEditDialogOpen]=useState(false); //外観検査機器の編集ダイアログの制御
  const isBackup = insp.is_backup === true; //バックアップを実施するかどうか

  //編集ボタンクリック時のハンドラ
  const handleEdit=(e)=>{
    e.stopPropagation();
    if(isBackupRunning){
      alert("バックアップ処理中は編集できません");
      return;
    }
    setIsEditDialogOpen(true);
  }

  //バックアップ設定変更ボタンクリック時のハンドラ(バックアップの実施有効・無効の切り替え)
  const handleChangeBackupSettings=async (e)=>{
    e.stopPropagation();
    if(isBackupRunning){
      alert("バックアップ処理中は切り替えできません");
      return;
    }

    try {
        //バックエンドで更新を実施
        const backend_insp_configs = await invoke("change_insp_backup_settings",{inspId:insp.id});
        console.log("backend_insp_configs",backend_insp_configs);

        //受け取ったbackup_insp_configsと現在のinspListを合体
        const new_insp_list=backend_insp_configs.map((backend_insp)=>{
            let last_backuped="-";
            inspList.forEach((current_insp)=>{
                if(backend_insp.id===current_insp.id) last_backuped=current_insp.lastBackuped;
            });
            return {
                ...backend_insp,
                lastBackuped: last_backuped
            };
        });

        //insplistを更新
        setInspList(new_insp_list);
        isBackup ? alert(`${insp.name}のバックアップの無効化が完了しました`) : alert(`${insp.name}のバックアップの有効化が完了しました`)
    } catch (error) {
        console.error("Failed to Edit insp info:", error);
        isBackup ? alert(`バックアップの無効化に失敗しました : ${error}`) : alert(`バックアップの有効化に失敗しました : ${error}`)
    }
  }

  //検査装置を削除
  const handleDeleteSettings=async (e)=>{
    e.stopPropagation();
    if(isBackupRunning){
      alert("バックアップ処理中は切り替えできません");
      return;
    }

    const result = await ask(`${insp.name}を本当に削除しますか？`, {
      title: "削除の確認",
      kind: "warning"
    });
    if (!result) {
      return;
    }

    try {
        //バックエンドで更新を実施
        const backend_insp_configs = await invoke("delete_insp_configs",{id:insp.id});
        console.log("delete_insp_configs",backend_insp_configs);

        //受け取ったbackup_insp_configsと現在のinspListを合体
        const new_insp_list=backend_insp_configs.map((backend_insp)=>{
            let last_backuped="-";
            inspList.forEach((current_insp)=>{
                if(backend_insp.id===current_insp.id) last_backuped=current_insp.lastBackuped;
            });
            return {
                ...backend_insp,
                lastBackuped: last_backuped
            };
        });

        //insplistを更新
        setInspList(new_insp_list);
        alert(`${insp.name}の削除が完了しました`);
    } catch (error) {
        console.error("Failed to Delete insp info:", error);
        alert(`${insp.name}の削除に失敗しました : ${error}`);
    }
  }

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
              <p className="text-black font-mono">{insp.insp_ip}</p>
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
                onClick={(e)=>handleEdit(e)}
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
                onClick={(e)=>handleChangeBackupSettings(e)}
                className={`w-full flex items-center justify-center gap-2 px-4 py-1 rounded-lg transition-colors ${
                  isBackupRunning
                    ? "bg-gray-700 text-gray-500 cursor-not-allowed"
                    : isBackup ? "bg-orange-700 hover:bg-orange-600 text-white" : "bg-blue-700 hover:bg-blue-600 text-white"
                }`}
              >
                <Cog size={16} />
                {isBackupRunning ? "バックアップ処理中は編集できません" : isBackup ? "この検査装置のバックアップを停止": "この検査装置のバックアップを再開"}
              </button>
            </div>

            {/* 削除ボタン */}
            <div className="border-t border-gray-700 pt-4">
              <button
                disabled={isBackupRunning}
                onClick={(e)=>handleDeleteSettings(e)}
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
      {/* 外観検査機器情報編集ダイアログ */}
      <EditInspDialog
        insp={insp}
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
      />
    </div>
  );
}
