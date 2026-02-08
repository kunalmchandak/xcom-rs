# separate-execution-context Proposal

## 背景
`ExecutionContext` がバリデーションロジック（`check_interaction_required`, `check_max_cost`, `check_daily_budget`）を持ち、単一責任原則に反しています。

## 目的
実行コンテキストとビジネスルールを分離し、テスト容易性と保守性を向上させます。

## スコープ
- `ExecutionContext` を純粋なデータホルダーに変更
- バリデーションロジックを別の型（例: `ExecutionPolicy` / `Validator`）に移動

## 非スコープ
- バリデーションロジックの仕様変更
- 新しいバリデーションルールの追加

## 成果物
- 責務が分離された `ExecutionContext` と `ExecutionPolicy`
- 既存テストの更新

## リスク/留意点
- 呼び出し側の変更が広範囲に及ぶ可能性
