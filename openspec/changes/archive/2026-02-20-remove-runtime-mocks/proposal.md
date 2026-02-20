## Why

`tweets list` がダミー生成を返してしまう経路が残っており、本番実行でモック実装に到達するリスクがある。実運用の信頼性と誤動作防止のため、モック実装を完全に撤廃する必要がある。

## What Changes

- `tweets` 実行経路のモッククライアントとダミー生成を削除し、実API応答のみを返す
- `XCOM_SIMULATE_ERROR` による擬似エラー注入を削除し、エラーはAPIレスポンスに基づいて返す
- テストは `mockito` を用いたHTTPモックに統一し、外部API依存を排除する
- テストユーティリティはテスト専用領域へ移動し、本番ビルドから分離する

## Capabilities

### New Capabilities

- なし

### Modified Capabilities

- `tweets-query-efficiency`: `tweets list` が実API結果のみを返し、モック生成を禁止する
- `tweets-replies`: `tweets show/conversation` が実API結果のみを返し、モックフォールバックを禁止する
- `timeline-reads`: タイムライン取得はAPI結果に基づき、擬似エラー注入を禁止する

## Impact

- 影響範囲: `src/tweets/client.rs`, `src/tweets/commands/*`, `src/handlers/tweets.rs`, `src/timeline/commands.rs`, `src/lib.rs`, `tests/`
- 環境変数: `XCOM_SIMULATE_ERROR`, `XCOM_RETRY_AFTER_MS` を廃止
- テスト: `mockito` を利用したHTTPスタブへの置換が必要
