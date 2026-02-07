# auth-headless Specification

## Purpose
TBD - created by archiving change plan-headless-auth-and-billing. Update Purpose after archive.
## Requirements
### Requirement: 非対話認証状態の取得
`xcom-rs` は `auth status --output json` で、エージェントが認証前提条件を判断できる情報を返さなければならない（MUST）。

#### Scenario: 未認証時の次アクション提示
- **Given** 利用者環境に有効な認証情報がない
- **When** `auth status --output json` を実行する
- **Then** `authenticated=false` と `nextSteps` が返る
- **And** エラーメッセージではなく判定可能な状態情報として返る

### Requirement: 認証情報のヘッドレス移送
`xcom-rs` は `auth export` と `auth import` を提供し、対話なしで認証状態を移送できなければならない（MUST）。

#### Scenario: export/importによる復元
- **Given** 認証済み環境で `auth export` を実行した結果がある
- **When** 別環境で `auth import` を実行する
- **Then** `auth status --output json` が `authenticated=true` を返す

### Requirement: 非対話モードでの安全失敗
`--non-interactive` 指定時、認証が不足している操作はプロンプトを出さずに構造化エラーで失敗しなければならない（MUST）。

#### Scenario: 非対話時の認証不足
- **Given** 未認証状態で認証必須操作を `--non-interactive` で実行する
- **When** CLIが失敗を返す
- **Then** `error.code=auth_required` と `nextSteps` を返す
- **And** 終了コード `3` を返す

