## Why

現在の `xcom-rs` は `XCOM_RS_BEARER_TOKEN` の環境変数のみを認証情報として扱うため、ユーザに紐づく OAuth2.0 user access token の取得フローが存在しません。これにより `/2/users/me` などユーザコンテキスト必須のAPIが403になり、CLIの主要機能が利用できない状態が発生します。

## What Changes

- OAuth2.0 Authorization Code + PKCE による `auth login` を追加し、user access token と refresh token を取得・保存できるようにする
- 保存済みトークンを優先度ルールに基づいて解決し、X API クライアントがそれを利用する
- `auth status` が保存済みトークンの状態（期限、スコープ、更新可否）を返せるようにする
- `auth logout` により保存済み認証情報を削除し、必要に応じてrevokeも行えるようにする

## Capabilities

### New Capabilities
- `auth-oauth2-login`: OAuth2.0 Authorization Code + PKCE によるユーザ認証取得・保存・更新の機能

### Modified Capabilities
- `auth-headless`: `auth status` が保存済みのOAuth2認証情報も考慮して状態を返す
- `x-api-client`: X API呼び出し時のトークン解決が環境変数だけでなく保存済み認証情報にも対応する

## Impact

- 影響コード: `src/auth/*`, `src/handlers/auth.rs`, `src/cli.rs`, `src/x_api/*`, `src/doctor.rs`
- 外部依存: X OAuth2.0 エンドポイント（authorize/token/refresh/revoke）
- 設定/保存先: `dirs::config_dir()` 配下の `xcom-rs/auth.json`
