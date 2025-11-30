import React, { useState,useEffect } from "react";
import { X } from "lucide-react";
import { useNASContext } from "../contexts/NASContext";
import { invoke } from "@tauri-apps/api/core";

/**
 * 追加ダイアログコンポーネント
 * @param {Object} props
 * @param {Object} porps.insp
 * @param {boolean} props.isOpen - ダイアログの開閉状態
 * @param {Function} props.onClose - 閉じるハンドラー
 */
export default function EditInspDialog({ insp,isOpen, onClose}) {
  const [formData, setFormData] = useState({
    id:insp.id,
    name: insp.name,
    insp_ip: insp.insp_ip,
    surface_image_path: insp.surface_image_path,
    back_image_path: insp.back_image_path,
    result_path: insp.result_path,
    is_backup: insp.is_backup,
  });
const { inspList,setInspList } = useNASContext(); // グローバルなNAS・外観検査機一覧

  useEffect(() => {
    if (insp) {
      setFormData({
        id:insp.id,
        name: insp.name,
        insp_ip: insp.insp_ip,
        surface_image_path: insp.surface_image_path,
        back_image_path: insp.back_image_path,
        result_path: insp.result_path,
        is_backup: insp.is_backup,
      });
    }
  }, [insp]);

  //バックエンドと通信
  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
        //バックエンドで更新を実施
        const backend_insp_configs = await invoke("edit_insp_configs",{newInspInfo:formData});
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
        onClose();
        alert("外観検査情報の更新が完了しました");
    } catch (error) {
        console.error("Failed to Edit insp info:", error);
        alert(`外観検査情報の編集に失敗しました : ${error}`);
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
                <label className="block text-sm text-gray-700 mb-1">装置名</label>
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
                <label className="block text-sm text-gray-700 mb-1">外観検査IPアドレス</label>
                <input
                type="text"
                value={formData.insp_ip}
                onChange={(e) => setFormData({ ...formData, insp_ip: e.target.value })}
                className="w-full px-3 py-2 bg-gray-100 text-black rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="例: 192.168.1.100"
                required
                />
            </div>

            <div>
                <label className="block text-sm text-gray-700 mb-1">外観検査表面画像フォルダパス</label>
                <input
                type="text"
                value={formData.surface_image_path}
                onChange={(e) => setFormData({ ...formData, surface_image_path: e.target.value })}
                className="w-full px-3 py-2 bg-gray-100 text-black rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="/home/usr/jpg/surface"
                required
                />
            </div>

            <div>
                <label className="block text-sm text-gray-700 mb-1">外観検査裏面画像フォルダパス</label>
                <input
                type="text"
                value={formData.back_image_path}
                onChange={(e) => setFormData({ ...formData, back_image_path: e.target.value })}
                className="w-full px-3 py-2 bg-gray-100 text-black rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="/home/usr/jpg/back"
                required
                />
            </div>

            <div>
                <label className="block text-sm text-gray-700 mb-1">外観検査resultファイルフォルダパス</label>
                <input
                type="text"
                value={formData.result_path}
                onChange={(e) => setFormData({ ...formData, result_path: e.target.value })}
                className="w-full px-3 py-2 bg-gray-100 text-black rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="/home/usr/result"
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
              className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              編集
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}