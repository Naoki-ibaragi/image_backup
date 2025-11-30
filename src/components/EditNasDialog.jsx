import React, { useState,useEffect } from "react";
import { X } from "lucide-react";
import { useNASContext } from "../contexts/NASContext";
import { invoke } from "@tauri-apps/api/core";

/**
 * 追加ダイアログコンポーネント
 * @param {Object} props
 * @param {Object} porps.nas
 * @param {boolean} props.isOpen - ダイアログの開閉状態
 * @param {Function} props.onClose - 閉じるハンドラー
 */
export default function EditNasDialog({ nas,isOpen, onClose}) {
  const [formData, setFormData] = useState({
    id:nas.id,
    name: nas.name,
    nas_ip: nas.nas_ip,
    drive: nas.drive,
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
const { nasList,setNasList } = useNASContext(); // グローバルなNAS・外観検査機一覧

  useEffect(() => {
    if (nas) {
      setFormData({
        id:nas.id,
        name: nas.name,
        nas_ip: nas.nas_ip,
        drive: nas.drive,
      });
    }
  }, [nas]);

  //バックエンドと通信
  const handleSubmit = async (e) => {
    e.preventDefault();
    if (isSubmitting) return; // 二重送信防止

    setIsSubmitting(true);
    try {
        //バックエンドで更新を実施
        const backend_nas_configs = await invoke("edit_nas_configs",{newNasInfo:formData});
        console.log("backend_nas_configs",backend_nas_configs);

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
        onClose();
        alert("NAS情報の更新が完了しました");
    } catch (error) {
        console.error("Failed to Edit nas info:", error);
        alert(`NAS情報の編集に失敗しました : ${error}`);
    } finally {
        setIsSubmitting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-100 rounded-lg p-6 w-full max-w-md shadow-xl">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-bold font-mono text-black">外観検査機器情報を編集</h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
          >
            <X size={24} className="text-gray-400" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
            <div>
                <label className="block text-sm text-gray-700 mb-1">NAS名称</label>
                <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full px-3 py-2 bg-gray-100 text-black rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="例: CLT5"
                required
                />
            </div>

            <div>
                <label className="block text-sm text-gray-400 mb-1">NAS IPアドレス</label>
                <input
                type="text"
                value={formData.nas_ip}
                onChange={(e) => setFormData({ ...formData, nas_ip: e.target.value })}
                className="w-full px-3 py-2 bg-gray-100 text-black rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="例: 192.168.1.100"
                required
                />
            </div>

            <div>
                <label className="block text-sm text-gray-400 mb-1">NAS ネットワークドライブ名</label>
                <input
                type="text"
                value={formData.drive}
                onChange={(e) => setFormData({ ...formData, drive: e.target.value })}
                className="w-full px-3 py-2 bg-gray-100 text-black rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="A"
                required
                />
            </div>
          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
            >
              キャンセル
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting ? "処理中..." : "編集"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}