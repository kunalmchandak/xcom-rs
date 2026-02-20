# auth-headless Specification

## Purpose
TBD - created by archiving change plan-headless-auth-and-billing. Update Purpose after archive.
## Requirements
### Requirement: 非対話認証状態の取得
`xcom-rs` は `auth status --output json` で、エージェントが認証前提条件を判断できる情報を返さなければならない（MUST）。

#### Scenario: 未認証時の次アクション提示
- **Given** 利用者環境に `XCOM_RS_BEARER_TOKEN` が設定されていない
- **When** `auth status --output json` を実行する
- **Then** `authenticated=false` と `nextSteps` が返る
- **And** `nextSteps` に環境変数設定案内が含まれる
- **And** エラーメッセージではなく判定可能な状態情報として返る

### Requirement: 認証情報の環境変数による設定
`xcom-rs` は環境変数 `XCOM_RS_BEARER_TOKEN` から認証情報を読み取り、対話なしで認証状態を確立できなければならない（MUST）。

#### Scenario: 環境変数による認証
- **Given** `XCOM_RS_BEARER_TOKEN` に有効なトークンが設定されている
- **When** `auth status --output json` を実行する
- **Then** `authenticated=true` を返す

### Requirement: 非対話モードでの安全失敗
`--non-interactive` 指定時、認証が不足している操作はプロンプトを出さずに構造化エラーで失敗しなければならない（MUST）。

#### Scenario: 非対話時の認証不足
- **Given** 未認証状態で認証必須操作を `--non-interactive` で実行する
- **When** CLIが失敗を返す
- **Then** `error.code=auth_required` と `nextSteps` を返す
- **And** `nextSteps` に環境変数設定案内が含まれる
- **And** 終了コード `3` を返す

