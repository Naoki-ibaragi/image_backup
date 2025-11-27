import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useNASContext } from '../contexts/NASContext';

/**
 * リスナーを初期化するカスタムフック
 */
export const useInitListener = () => {
    const { setNasList } = useNASContext();

    useEffect(() => {
        let unlistenMessage;
        let unlistenNasStatus;

        const setupListeners = async () => {
            try {
                // nas-messageリスナー
                unlistenMessage = await listen('nas-message', (event) => {
                    const { nas_id, message, timestamp } = event.payload;

                    // nullや空データの場合は更新しない(切断時に無効なデータが送られてくる場合があるため)
                    if (!message || message === "" || (typeof message === 'object' && Object.keys(message).length === 0)) {
                        console.log(`NAS ${nas_id}: 空またはnullのデータを受信したためスキップ`);
                        return;
                    }

                    //対象のnas_idのnasListのlastReceivedをmessageで更新
                    setNasList((prev) =>
                        prev.map((p) =>
                            p.id === nas_id
                                ? { ...p, lastReceived: timestamp, data: message }
                                : p
                        )
                    );
                });

                // nas-status-updatedリスナー
                unlistenNasStatus = await listen('nas-status-updated', (event) => {
                    const nas_configs = event.payload;

                    // nas_configsでnasListを更新
                    setNasList((prev) =>
                        prev.map((nas) => {
                            const updatedConfig = nas_configs.find(config => config.id === nas.id);
                            if (updatedConfig) {
                                return {
                                    ...nas,
                                    is_connected: updatedConfig.is_connected,
                                    is_use: updatedConfig.is_use,
                                    total_space: updatedConfig.total_space,
                                    used_space: updatedConfig.used_space,
                                    free_space: updatedConfig.free_space,
                                };
                            }
                            return nas;
                        })
                    );
                });

            } catch (err) {
                console.error("Failed to setup listener:", err);
            }
        };

        setupListeners();

        // クリーンアップ関数
        return () => {
            if (unlistenMessage) {
                unlistenMessage();
            }
            if (unlistenNasStatus) {
                unlistenNasStatus();
            }
        };
    }, [setNasList]);
};
