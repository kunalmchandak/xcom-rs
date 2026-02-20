## Why

現在のCLIはX API呼び出し部分がスタブ・モックに依存しており、実運用に必要なHTTPクライアントと共通エラー分類が欠けています。実API実装に移行するための共通基盤を先に整備します。

## What Changes

- X API向けの共通HTTPクライアント（ベースURL/認証/ヘッダー/JSON入出力）を追加する
- 429/5xx/認証失敗などのレスポンスを既存のエラー表現にマッピングする
- テスト用のモック実装とモックサーバ接続を用意する

## Capabilities

### New Capabilities
- `x-api-client`: X API通信の共通クライアントとエラー分類の基盤

### Modified Capabilities
- なし

## Impact

- `src/` にX APIクライアントの新規モジュールを追加
- 新規HTTP依存の追加（同期HTTP）
- 後続変更（読み取り/書き込みの実API化）から参照される共通基盤になる
