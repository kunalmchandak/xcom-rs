# billing-guardrails Specification

## Purpose
TBD - created by archiving change plan-headless-auth-and-billing. Update Purpose after archive.
## Requirements
### Requirement: 実行前コスト見積
`xcom-rs` は `billing estimate` で、設定されたレート表に基づいて見積を返さなければならない（MUST）。

#### Scenario: 設定レート表の適用
- **Given** レート表設定ファイルに `tweets.create=7` が定義されている
- **When** 利用者が `billing estimate tweets.create --text "hello"` を実行する
- **Then** `cost.credits` は設定値に基づく

### Requirement: 単発コスト上限ガード
`xcom-rs` は `--max-cost-credits` を超える見積の操作を、実行前に拒否しなければならない（MUST）。

#### Scenario: 上限超過の事前拒否
- **Given** 対象操作の見積コストが `5` クレジット
- **And** 利用者が `--max-cost-credits 3` を指定する
- **When** 操作を実行する
- **Then** `error.code=cost_limit_exceeded` を返し、外部API呼び出しを行わない

### Requirement: 日次予算ガード
`xcom-rs` は `--budget-daily-credits` を超える当日累積を検知し、追加実行を止めなければならない（MUST）。

#### Scenario: 当日予算超過
- **Given** 当日累積使用量が `9` クレジット
- **And** 利用者が `--budget-daily-credits 10` を指定する
- **When** 見積 `2` クレジットの操作を実行する
- **Then** `error.code=daily_budget_exceeded` を返し実行しない

### Requirement: dry-runの無課金保証
`xcom-rs` は `--dry-run` 指定時、外部API呼び出しなしで見積情報のみ返さなければならない（MUST）。

#### Scenario: dry-run実行
- **Given** 利用者が課金対象操作を `--dry-run --output json` で実行する
- **When** CLIが結果を返す
- **Then** `meta.dryRun=true` と `meta.cost.credits=0` を返す

### Requirement: 利用状況レポート
`xcom-rs` は `billing report` で当日の使用量を返さなければならない（MUST）。

#### Scenario: 当日使用量の返却
- **Given** 当日累積使用量が `12` クレジット
- **When** 利用者が `billing report --output json` を実行する
- **Then** `data.todayUsage=12` が返る

