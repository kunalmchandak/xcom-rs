## 1. レート表の実装

- [x] 1.1 レート表設定ファイルの読み込みを追加する（検証: `billing_rates.json` の読み込み処理が追加されている）
- [x] 1.2 `CostEstimator` が設定レートを優先して使用するようにする（検証: `CostEstimator::estimate` が設定値を参照する）

## 2. レポートの実装

- [x] 2.1 `billing report` が `BudgetTracker` の `today_usage` を返すようにする（検証: `src/handlers/billing.rs` の `todayUsage` が動的値になる）

## 3. テスト

- [x] 3.1 レート表の読み込み/フォールバックの単体テストを追加する（検証: `cargo test billing::` が成功する）
- [x] 3.2 `billing report` が当日使用量を返すテストを追加する（検証: `tests/auth_billing_test.rs` に期待値が追加される）
