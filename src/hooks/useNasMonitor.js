import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/**
 * NASの監視状態を管理するカスタムフック
 * @returns {Object} NAS設定の配列とローディング状態
 */
export function useNasMonitor() {
  const [nasConfigs, setNasConfigs] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    // 初回のNAS状態を取得
    const fetchNasStatus = async () => {
      try {
        const configs = await invoke('get_nas_status');
        setNasConfigs(configs);
        setLoading(false);
      } catch (err) {
        console.error('Failed to fetch NAS status:', err);
        setError(err);
        setLoading(false);
      }
    };

    fetchNasStatus();

    // 10秒ごとの更新イベントをリッスン
    const unlisten = listen('nas-status-updated', (event) => {
      console.log('NAS status updated:', event.payload);
      setNasConfigs(event.payload);
    });

    // クリーンアップ
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  /**
   * 特定のNASの転送状態を更新
   * @param {number} nasId - NASのID
   * @param {boolean} isTransfer - 転送中かどうか
   */
  const setTransferStatus = async (nasId, isTransfer) => {
    try {
      await invoke('set_nas_transfer_status', { nasId, isTransfer });
    } catch (err) {
      console.error('Failed to set transfer status:', err);
      throw err;
    }
  };

  return {
    nasConfigs,
    loading,
    error,
    setTransferStatus,
  };
}
