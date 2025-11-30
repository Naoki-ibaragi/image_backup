import React, { useState } from "react";
import { ChevronDown, CheckCircle, XCircle, Wifi, Database, Square, Cog, Trash2,Circle, CircleOff } from "lucide-react";
import { useNASContext } from "../contexts/NASContext";
import EditNasDialog from "./EditNasDialog";
import { invoke } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";

/**
 * 個別のNASカードコンポーネント
 * @param {Object} props
 * @param {Object} props.nas - NAS情報
 * @param {Object} props.config - ソケット情報
 */
export default function NASCard({nas}) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const isConnected = nas.is_connected === true;
  const { isBackupRunning,nasList,setNasList } = useNASContext(); // グローバルなNAS・外観検査機一覧
  const [isEditDialogOpen,setIsEditDialogOpen]=useState(false); //外観検査機器の編集ダイアログの制御

  const bytesToGB=(bytes)=>{
    return (bytes/1024/1024/1024).toFixed(2);
  }

  //NASを編集
  const handleEdit=(e)=>{
    e.stopPropagation();
    if(isBackupRunning){
      alert("バックアップ処理中は編集できません");
      return;
    }
    setIsEditDialogOpen(true);
  }

  //NASを削除
  const handleDeleteSettings=async (e)=>{
    e.stopPropagation();
    if(isBackupRunning){
      alert("バックアップ処理中は切り替えできません");
      return;
    }

    const result = await ask(`${nas.name}を本当に削除しますか？`, {
      title: "削除の確認",
      kind: "warning"
    });
    if (!result) {
      return;
    }

    try {
        //バックエンドで更新を実施
        const backend_nas_configs = await invoke("delete_nas_configs",{id:nas.id});
        console.log("delete_nas_configs",backend_nas_configs);

        //受け取ったbackup_nas_configsと現在のnasListを合体
        const new_nas_list=backend_nas_configs.map((backend_nas)=>{
            let is_use=true;
            let is_connected=false;
            let total_space=0;
            let used_space=0;
            let free_space=0;
            nasList.forEach((current_nas)=>{
                if(backend_nas.id===current_nas.id) {
                  is_use=current_nas.is_use;
                  is_connected=current_nas.is_use;
                  total_space=current_nas.total_space;
                  used_space=current_nas.used_space;
                  free_space=current_nas.free_space;
                }
            });
            return {
                ...backend_nas,
                is_use: is_use,
                is_connected: is_connected,
                total_space:total_space,
                used_space:used_space,
                free_space:free_space,
            };
        });

        //naslistを更新
        setNasList(new_nas_list);
        alert(`${nas.name}の削除が完了しました`);
    } catch (error) {
        console.error("Failed to Delete nas info:", error);
        alert(`${nas.name}の削除に失敗しました : ${error}`);
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
          <Database className={isConnected ? "text-blue-600" : "text-gray-500"} size={24} />
          <div className="flex-1 text-left">
            <h3 className="text-lg font-semibold text-black text-mono">{nas.name}</h3>
          </div>
          <span
            className={`flex items-center gap-1 px-3 py-1 rounded-full text-sm font-medium ${
              isConnected
                ? "bg-green-900/20 text-green-700"
                : "bg-red-900/20 text-red-600"
            }`}
          >
            {isConnected ? (
              <>
                <CheckCircle size={16} />
                接続中
              </>
            ) : (
              <>
                <XCircle size={16} />
                接続できません
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
        <div className="px-4 pb-4 border-t">
          <div className="pt-4 space-y-4">
            <div>
              <p className="text-sm text-gray-400 mb-1">IPアドレス</p>
              <p className="text-black font-mono">{nas.nas_ip}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">ネットワークドライブ</p>
              <p className="text-black font-mono">{nas.drive}\:</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">容量</p>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <p className="text-black font-mono text-sm">
                    {bytesToGB(nas.used_space)} GB / {bytesToGB(nas.total_space)} GB
                  </p>
                  <p className="text-black font-semibold">
                    {((nas.used_space / nas.total_space) * 100).toFixed(1)}%
                  </p>
                </div>
                {/* プログレスバー */}
                <div className="w-full bg-gray-300 rounded-full h-3 overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all ${
                      (nas.used_space / nas.total_space) * 100 > 90
                        ? "bg-red-600"
                        : (nas.used_space / nas.total_space) * 100 > 75
                        ? "bg-yellow-500"
                        : "bg-green-600"
                    }`}
                    style={{ width: `${(nas.used_space / nas.total_space) * 100}%` }}
                  ></div>
                </div>
              </div>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">最終ファイル更新時刻</p>
              <p className="text-black">{nas.lastReceived}</p>
            </div>

            <div className="border-t border-gray-700 pt-4">
              <p className="text-sm text-gray-400 mb-2">ログ</p>
              {nas.data ? (
                <div className="bg-gray-900 p-3 rounded border border-gray-700">
                  <pre className="text-sm text-green-400 font-mono overflow-y-auto max-h-20 whitespace-pre-wrap break-all">
                    {JSON.stringify(nas.data, null, 2)}
                  </pre>
                </div>
              ) : (
                <p className="text-gray-500 italic">データなし</p>
              )}
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
                {isBackupRunning ? "バックアップ処理中は編集できません" : "このNAS情報を編集"}
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
                {isBackupRunning ? "バックアップ処理中は削除できません" : "このNASを削除"}
              </button>
            </div>
          </div>
        </div>
      )}
      {/* 外観検査機器情報編集ダイアログ */}
      <EditNasDialog
        nas={nas}
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
      />
    </div>
  );
}
