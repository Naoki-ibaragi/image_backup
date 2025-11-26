import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { X, Plus } from "lucide-react";
import NASCard from "./NASCard";
import INSPCard from "./INSPCard";
import { listen } from '@tauri-apps/api/event';

export default function StackCard() {
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [nasList, setNasList] = useState([]); //LAN上のNAS一覧
    const [nasConfigs, setNasConfigs] = useState([]); // 元の設定データを保持
    const [inspList, setInspList] = useState([]); //LAN上の外観検査機一覧
    const [inspConfigs, setInspConfigs] = useState([]); // 元の設定データを保持
    const [tab,setTab]=useState("NAS");

    // アプリ起動時にNAS設定を読み込む
    useEffect(() => {
        const loadNasConfig = async () => {
            try {
                //バックエンドからNAS・外観検査一覧情報を取得(アプリ起動時のみ)
                const configs = await invoke("init_initial_info");

                setNasConfigs(configs[0]);
                const NasFormattedData = configs[0].map((config) => ({
                    id: config.id,
                    name: config.name,
                    ip: config.nas_ip,
                    drive: config.drive,
                    now_target: false,                      //現在転送先の対象か
                    next_target: false,                     //次に転送先の対象になるか
                    is_connected: config.is_connected,      //認識できているか
                    is_transofer: config.is_transfer,       //転送実施中かどうか
                    total_space: config.total_space,        //NASの全容量
                    current_space: config.current_space,    //NASの現在の容量
                    free_space: config.free_space,          //NASの現在の空容量
                    lastReceived: "-",
                    data: null,
                }));
                setNasList(NasFormattedData);

                setInspConfigs(configs[1]);
                const InspFormattedData = configs[1].map((config) => ({
                    id: config.id,
                    name: config.name,
                    ip: config.insp_ip,
                    drive: config.drive,
                    is_backup: config.is_backup,            //転送実施するかどうか
                    is_transfer:false,                      //転送中かどうか
                    lastBackuped: "-",
                }));
                setInspList(InspFormattedData);

                setLoading(false);
            } catch (err) {
                console.error("Failed to load config:", err);
                setError(err);
                setLoading(false);
            }
        };

        // listenハンドラの立上
        const openListener = async () => {
        try {
            const unlistenMessage = await listen('nas-message', (event) => {
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
                    ? { ...p, lastReceived: timestamp, data:message }
                    : p
                )
            );
            });

            const unlistenDisconnect = await listen('nas-disconnected', (event) => {
            const { nas_id, reason } = event.payload;
            console.log(`NAS ${nas_id} disconnected: ${reason}`);

            // 対象のNASの接続状態を切断に更新(最終受信データと時刻は保持)
            setNasList((prev) =>
                prev.map((p) =>
                p.id === nas_id
                    ? { ...p,is_connected: false }
                    : p
                )
            );

            //切断されたNASが
            });

            return () => {
            unlistenMessage();
            unlistenDisconnect();
            };
        } catch (err) {
            console.error("Failed to setup listener:", err);
            setError(err);
            setLoading(false);
            return null;
        }
        }

        //Nasの初期設定実施
        loadNasConfig();
        
        // クリーンアップ関数を返す
        let unlistenFn;
        openListener().then(fn => {
        unlistenFn = fn;
        });

        return () => {
            // コンポーネントのアンマウント時にリスナーを解除
            if (unlistenFn) {
                unlistenFn();
            }
        };
    }, []);

    const hideWindow = async () => {
        try {
        const appWindow = getCurrentWindow();
        await appWindow.hide();
        } catch (error) {
        console.error("Failed to hide window:", error);
        }
    };

    //接続されているnasの数
    const connectedCount = nasList.filter((nas) => nas.is_connected === true).length;

    //tab選択時のコールバック関数
    const switchTab=(e,tab_name)=>{
        e.preventDefault();
        if(tab_name==="NAS" && (tab==="INSP" || tab==="SETTINGS")){
            setTab("NAS");
        }else if(tab_name==="INSP" && (tab==="NAS" || tab==="SETTINGS")){
            setTab("INSP");
        }else if(tab_name==="SETTINGS" && (tab==="NAS" || tab==="INSP")){
            setTab("SETTINGS");
        }else{
            return;
        }
    } 


    // ローディング中の表示
    if (loading) {
        return (
        <div className="min-h-screen text-white flex items-center justify-center">
            <div className="text-center">
            <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-blue-400 mx-auto mb-4"></div>
            <p className="text-gray-400">NAS設定を読み込み中...</p>
            </div>
        </div>
        );
    }

    // エラー時の表示
    if (error) {
        return (
        <div className="min-h-screen text-white">
            <header className="bg-blue-700 shadow-lg">
                <div className="flex items-center justify-between p-2">
                    <h1 className="text-xl font-mono">Image Backup</h1>
                    <button
                    onClick={hideWindow}
                    className="p-2 hover:bg-gray-700 rounded-full transition-colors"
                    aria-label="ウィンドウを閉じる"
                    >
                    <X size={24} />
                    </button>
                </div>
            </header>
            <main className="p-6">
            <div className="bg-red-900/10 border border-red-500 rounded-lg p-4">
                <h2 className="text-xl font-semibold text-red-500 mb-2">設定ファイルの読み込みに失敗しました</h2>
                <p className="text-red-400">{error.toString()}</p>
            </div>
            </main>
        </div>
        );
    }

    return (
        <div className="min-h-screen text-white">
            {/* ヘッダー */}
            <header className="bg-blue-700 shadow-lg">
                <div className="flex items-center justify-between p-2">
                <h1 className="text-2xl font-bold">Image Backup App</h1>
                <div className="flex items-center gap-4">
                    <button
                    onClick={hideWindow}
                    className="p-2 hover:bg-gray-700 rounded-full transition-colors"
                    aria-label="ウィンドウを閉じる"
                    >
                        <X size={24} />
                    </button>
                </div>
                </div>
            </header>
            <div className="text-sm font-medium text-center border-b border-gray-300">
                <ul className="flex flex-wrap -mb-px">
                    <li className="me-2">
                        <a
                            href="#"
                            onClick={(e)=>switchTab(e,"NAS")}
                            className={`inline-block p-4 border-b-2 transition-all duration-200 ${
                                tab === "NAS"
                                    ? "border-blue-600 text-blue-600 font-semibold bg-blue-50"
                                    : "border-transparent text-gray-600 hover:text-blue-600 hover:bg-gray-100 hover:border-gray-300"
                            }`}
                        >
                            NAS PAGE
                        </a>
                    </li>
                    <li className="me-2">
                        <a
                            href="#"
                            onClick={(e)=>switchTab(e,"INSP")}
                            className={`inline-block p-4 border-b-2 transition-all duration-200 ${
                                tab === "INSP"
                                    ? "border-blue-600 text-blue-600 font-semibold bg-blue-50"
                                    : "border-transparent text-gray-600 hover:text-blue-600 hover:bg-gray-100 hover:border-gray-300"
                            }`}
                        >
                            INSP PAGE
                        </a>
                    </li>
                    <li className="me-2">
                        <a
                            href="#"
                            onClick={(e)=>switchTab(e,"SETTINGS")}
                            className={`inline-block p-4 border-b-2 transition-all duration-200 ${
                                tab === "SETTINGS"
                                    ? "border-blue-600 text-blue-600 font-semibold bg-blue-50"
                                    : "border-transparent text-gray-600 hover:text-blue-600 hover:bg-gray-100 hover:border-gray-300"
                            }`}
                        >
                            SETTINGS PAGE
                        </a>
                    </li>
                </ul>
            </div>
            {/* メインコンテンツ */}
            <main className="p-6">
                {tab === "NAS" ? (
                    <>
                        <div className="flex items-center justify-between mb-4">
                            <div className="flex gap-4">
                                <h2 className="text-xl text-black font-semibold font-mono">NAS一覧</h2>
                                {connectedCount===0 ? <p className="text-lg text-red-600 font-mono">バックアップ可能なNASがありません</p> : null }
                            </div>
                            <button
                                className="flex items-center gap-2 px-4 py-2 bg-green-700 hover:bg-green-600 text-white rounded-lg transition-colors"
                            >
                                <Plus size={20} />
                                NAS追加
                            </button>
                        </div>

                        <div className="space-y-2">
                        {nasList.length > 0 ? (
                            nasList.map((nas,index) => (
                            <NASCard
                                key={nas.id}
                                nas={nas}
                            />
                            ))
                        ) : (
                            <p className="text-gray-400 text-center py-8">NASが登録されていません</p>
                        )
                        }
                        </div>
                    </>
                ) : tab === "INSP" ? (
                    <>
                        <div className="flex items-center justify-between mb-4">
                            <div className="flex gap-4">
                                <h2 className="text-xl text-black font-semibold font-mono">外観検査一覧</h2>
                            </div>
                            <button
                                className="flex items-center gap-2 px-4 py-2 bg-green-700 hover:bg-green-600 text-white rounded-lg transition-colors"
                            >
                                <Plus size={20} />
                                外観検査機追加
                            </button>
                        </div>

                        <div className="space-y-2">
                        {inspList.length > 0 ? (
                            inspList.map((insp,index) => (
                            <INSPCard
                                key={insp.id}
                                insp={insp}
                            />
                            ))
                        ) : (
                            <p className="text-gray-400 text-center py-8">外観検査機がが登録されていません</p>
                        )
                        }
                        </div>
                    </>
                ) : (
                    <></>
                )}
            </main>

        </div>
    );
}
