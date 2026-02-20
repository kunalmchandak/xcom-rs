## Why

現在のコスト見積は固定のスタブレート表に依存し、`billing report` も常に0を返しています。実際の使用状況に基づいた見積とレポートを提供する必要があります。

## What Changes

- コストレート表を設定ファイルから読み込み、見積に反映する
- `billing report` が実際の当日使用量を返すようにする
- レート表が無い場合のフォールバック挙動を明確化する

## Capabilities

### New Capabilities
- なし

### Modified Capabilities
- `billing-guardrails`: 見積レートの実体化と使用量レポートを追加

## Impact

- `src/billing/storage.rs` の `CostEstimator` と `BudgetTracker` の利用変更
- `src/handlers/billing.rs` の `report` 実装変更
