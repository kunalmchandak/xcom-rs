# separate-execution-context Design

## 設計方針
- `ExecutionContext` は実行時の設定値のみを保持する。
- バリデーションロジックは `ExecutionPolicy` に集約する。

## 構成案
- `ExecutionPolicy` は `ExecutionContext` を受け取り、各種チェックを実行する。
- `check_interaction_required`, `check_max_cost`, `check_daily_budget` を `ExecutionPolicy` のメソッドとする。

## 代替案
- `ExecutionContext` に trait を実装する案
  - 責務分離が曖昧になり、テストが複雑化する。
