# NAS Monitor 使用方法

## 概要

NAS Monitorは、複数のNASデバイスの接続状態と容量情報を10秒ごとに自動監視するシステムです。

## アーキテクチャ

### バックエンド (Rust/Tauri)

1. **NasMonitor構造体** ([nas_monitor.rs](src-tauri/src/nas_monitor.rs))
   - `Arc<RwLock<Vec<NasConfig>>>`でスレッドセーフにNAS情報を管理
   - 10秒ごとにNASの接続状態と容量をチェック
   - フロントエンドに`nas-status-updated`イベントを送信

2. **監視内容**
   - **接続チェック**: TCP接続でSMBポート(445)への到達性を確認
   - **容量取得**: 接続可能な場合、ドライブの総容量・使用量・空き容量を取得
   - **状態更新**: 10秒ごとに自動更新し、フロントエンドに通知

3. **提供されるコマンド**
   - `get_nas_status()`: 現在のNAS状態を取得
   - `set_nas_transfer_status(nas_id, is_transfer)`: 転送状態を更新

### フロントエンド (React)

1. **useNasMonitor Hook** ([useNasMonitor.js](src/hooks/useNasMonitor.js))
   - NAS状態の自動更新を管理
   - イベントリスナーで10秒ごとの更新を受信
   - 転送状態の更新機能を提供

## 使用例

### Reactコンポーネントでの使用

```jsx
import { useNasMonitor } from '../hooks/useNasMonitor';

function NasMonitorComponent() {
  const { nasConfigs, loading, error, setTransferStatus } = useNasMonitor();

  if (loading) return <div>読み込み中...</div>;
  if (error) return <div>エラー: {error}</div>;

  return (
    <div>
      {nasConfigs.map((nas) => (
        <div key={nas.id} className="nas-card">
          <h3>{nas.name}</h3>
          <p>ドライブ: {nas.drive}</p>
          <p>IPアドレス: {nas.nas_ip}</p>
          <p>接続状態: {nas.is_connected ? '接続中' : '未接続'}</p>
          <p>転送中: {nas.is_transfer ? 'はい' : 'いいえ'}</p>

          {nas.is_connected && (
            <div>
              <p>総容量: {formatBytes(nas.total_space)}</p>
              <p>使用中: {formatBytes(nas.current_space)}</p>
              <p>空き容量: {formatBytes(nas.free_space)}</p>
            </div>
          )}

          <button onClick={() => setTransferStatus(nas.id, true)}>
            転送開始
          </button>
        </div>
      ))}
    </div>
  );
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}
```

### 直接コマンドを使用する場合

```javascript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// NAS状態を取得
const nasConfigs = await invoke('get_nas_status');

// 転送状態を更新
await invoke('set_nas_transfer_status', {
  nasId: 1,
  isTransfer: true
});

// イベントリスナーを設定
const unlisten = await listen('nas-status-updated', (event) => {
  console.log('NAS状態が更新されました:', event.payload);
});

// クリーンアップ
unlisten();
```

## データ構造

### NasConfig

```rust
pub struct NasConfig {
    pub id: u32,              // NAS ID
    pub name: String,          // NAS名
    pub drive: String,         // ドライブレター (例: "Z:")
    pub nas_ip: String,        // IPアドレス
    pub is_connected: bool,    // 接続状態
    pub is_transfer: bool,     // 転送中フラグ
    pub total_space: u64,      // 総容量 (バイト)
    pub current_space: u64,    // 使用中容量 (バイト)
    pub free_space: u64,       // 空き容量 (バイト)
}
```

## 設定ファイル

NAS情報は`config.json`で管理されます:

```json
{
  "nas_units": {
    "nass": [
      {
        "id": 1,
        "name": "NAS1",
        "drive": "Z:",
        "nas_ip": "192.168.1.100"
      },
      {
        "id": 2,
        "name": "NAS2",
        "drive": "Y:",
        "nas_ip": "192.168.1.101"
      }
    ]
  }
}
```

## 注意事項

1. **接続タイムアウト**: NAS接続チェックは3秒でタイムアウトします
2. **監視間隔**: 10秒ごとに自動更新されます
3. **スレッドセーフ**: `Arc<RwLock>`により複数スレッドから安全にアクセス可能
4. **イベント駆動**: フロントエンドはイベントで自動更新されるため、ポーリング不要

## トラブルシューティング

### NASが接続できない場合
- ファイアウォールでポート445が開いているか確認
- NASのIPアドレスが正しいか確認
- ネットワーク接続を確認

### 容量が0と表示される場合
- ドライブがマウントされているか確認
- Windowsのドライブレター表記が正しいか確認 (例: "Z:" または "Z:\\")
