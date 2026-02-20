# auth-headless 仕様変更

## MODIFIED Requirements
### Requirement: 非対話認証状態の取得
`xcom-rs` は `auth status --output json` で、環境変数から取得した認証状態を返さなければならない（MUST）。

#### Scenario: 未認証時の次アクション提示
- **Given** `XCOM_RS_BEARER_TOKEN` が設定されていない
- **When** `auth status --output json` を実行する
- **Then** `authenticated=false` と `nextSteps` が返る
- **And** `nextSteps` は `XCOM_RS_BEARER_TOKEN` の設定を案内する
- **And** エラーメッセージではなく判定可能な状態情報として返る

### Requirement: 非対話モードでの安全失敗
`--non-interactive` 指定時、認証が不足している操作はプロンプトを出さずに構造化エラーで失敗しなければならない（MUST）。

#### Scenario: 非対話時の認証不足
- **Given** `XCOM_RS_BEARER_TOKEN` が設定されていない
- **When** 認証必須操作を `--non-interactive` で実行する
- **Then** `error.code=auth_required` と `nextSteps` を返す
- **And** `nextSteps` は `XCOM_RS_BEARER_TOKEN` の設定を案内する
- **And** 終了コード `3` を返す
