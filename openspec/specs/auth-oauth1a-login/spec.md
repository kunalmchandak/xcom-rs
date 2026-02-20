# auth-oauth1a-login Specification

## Purpose
TBD - created by archiving change add-oauth1a-auth. Update Purpose after archive.
## Requirements
### Requirement: OAuth1.0a ログインフローの提供
`xcom-rs` は OAuth1.0a 3-legged フローで user access token と token secret を取得できる `auth login --mode oauth1a` を提供しなければならない（MUST）。

#### Scenario: ローカルコールバックでのログイン
- **Given** 利用者が `xcom-rs auth login --mode oauth1a --method local-server` を実行する
- **When** CLIが request token を取得し、認可URLでユーザーが承認する
- **Then** `oauth_verifier` を使って access token と token secret を取得する
- **And** 認証情報が保存される

### Requirement: OAuth1.0a 認証情報の保存と表示
`xcom-rs` は OAuth1.0a の認証情報を `auth.json` に保存し、`auth status` に `authMode=oauth1a` を表示しなければならない（MUST）。

#### Scenario: 保存済みOAuth1.0aの状態表示
- **Given** `auth.json` に OAuth1.0a 認証情報が保存されている
- **When** `xcom-rs auth status --output json` を実行する
- **Then** `authenticated=true` を返す
- **And** `authMode=oauth1a` を返す
- **And** `refreshable=false` を返す

### Requirement: OAuth1.0a 環境変数による認証解決
`xcom-rs` は OAuth1.0a の環境変数が設定されている場合、保存済み認証情報より優先して利用しなければならない（MUST）。

#### Scenario: 環境変数での認証解決
- **Given** `XCOM_RS_OAUTH1_CONSUMER_KEY` などのOAuth1.0a環境変数が設定されている
- **When** APIリクエストの認証情報を解決する
- **Then** OAuth1.0aの認証情報が採用される

### Requirement: 非対話時の安全失敗(OAuth1.0a)
`xcom-rs` は `auth login --mode oauth1a` を `--non-interactive` で実行した場合、プロンプトを出さずに構造化エラーで失敗しなければならない（MUST）。

#### Scenario: 非対話でのOAuth1.0aログイン拒否
- **Given** 利用者が `xcom-rs --non-interactive auth login --mode oauth1a` を実行する
- **When** CLIが認証フローを開始しようとする
- **Then** `error.code=auth_required` と `nextSteps` を返す

### Requirement: OAuth1.0a ログアウトの失効
`xcom-rs` は `auth logout --revoke` 実行時、OAuth1.0a 認証情報が保存されている場合に失効エンドポイントを呼び出さなければならない（MUST）。

#### Scenario: OAuth1.0a トークンの失効
- **Given** `auth.json` に OAuth1.0a 認証情報が保存されている
- **When** 利用者が `xcom-rs auth logout --revoke` を実行する
- **Then** `oauth/invalidate_token` を呼び出す

