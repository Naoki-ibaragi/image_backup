import { createContext, useContext, useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

/**
 * NASリストを管理するContext
 */
const NASContext = createContext();

/**
 * NASContextを使用するカスタムフック
 * @returns {Object} nasList, setNasList, inspList, setInspList
 */
export const useNASContext = () => {
  const context = useContext(NASContext);
  if (!context) {
    throw new Error('useNASContext must be used within NASProvider');
  }
  return context;
};

/**
 * NASContextのProvider
 */
export const NASProvider = ({ children }) => {
  const [nasList, setNasList] = useState([]); // LAN上のNAS一覧
  const [inspList, setInspList] = useState([]); // LAN上の外観検査機一覧
  const [backupStartTime, setBackupStartTime] = useState('00:00') //バックアップ時刻設定
  const [surfaceImageFolderPath, setSurfaceImageFolderPath] = useState('') //NAS内表面画像保存パス設定
  const [backImageFolderPath, setBackImageFolderPath] = useState('') //NAS内裏面画像保存パス設定
  const [resultFolderPath, setResultFolderPath] = useState('') //検査結果保存パス設定

  // バックアップ状態管理
  const [isBackupRunning, setIsBackupRunning] = useState(false)
  const [backupProgress, setBackupProgress] = useState(null)
  const [lastBackupDate, setLastBackupDate] = useState(null)

  // バックアップイベントをリッスン
  useEffect(() => {
    let unlistenStarted, unlistenProgress, unlistenCompleted, unlistenFailed

    const setupListeners = async () => {
      // バックアップ開始
      unlistenStarted = await listen('backup-started', (event) => {
        console.log('Backup started:', event.payload)
        setIsBackupRunning(true)
        setBackupProgress(null)
      })

      // バックアップ進捗
      unlistenProgress = await listen('backup-progress', (event) => {
        console.log('Backup progress:', event.payload)
        setBackupProgress(event.payload)
      })

      // バックアップ完了
      unlistenCompleted = await listen('backup-completed', (event) => {
        console.log('Backup completed:', event.payload)
        setIsBackupRunning(false)
        setBackupProgress(null)
        setLastBackupDate(new Date().toISOString())
      })

      // バックアップ失敗
      unlistenFailed = await listen('backup-failed', (event) => {
        console.error('Backup failed:', event.payload)
        setIsBackupRunning(false)
        setBackupProgress(null)
        alert(`バックアップに失敗しました: ${event.payload}`)
      })
    }

    setupListeners()

    return () => {
      // クリーンアップ
      if (unlistenStarted) unlistenStarted()
      if (unlistenProgress) unlistenProgress()
      if (unlistenCompleted) unlistenCompleted()
      if (unlistenFailed) unlistenFailed()
    }
  }, [])

  return (
    <NASContext.Provider
      value={{
        nasList,
        setNasList,
        inspList,
        setInspList,
        backupStartTime,
        setBackupStartTime,
        surfaceImageFolderPath,
        setSurfaceImageFolderPath,
        backImageFolderPath,
        setBackImageFolderPath,
        resultFolderPath,
        setResultFolderPath,
        isBackupRunning,
        backupProgress,
        lastBackupDate
      }}>
      {children}
    </NASContext.Provider>
  );
};
