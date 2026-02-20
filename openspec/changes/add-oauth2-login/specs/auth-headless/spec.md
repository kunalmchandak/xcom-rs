# auth-headless Specification (Delta)

## MODIFIED Requirements
### Requirement: 認証情報の環境変数による設定
`xcom-rs` は環境変数 `XCOM_RS_BEARER_TOKEN` を優先しつつ、保存済みの OAuth2 認証情報からも認証状態を確立できなければならない（MUST）。

#### Scenario: 保存済みトークンによる認証
- **Given** `XCOM_RS_BEARER_TOKEN` が設定されていない
- **And** `auth.json` に有効なアクセストークンが保存されている
- **When** `auth status --output json` を実行する
- **Then** `authenticated=true` を返す
- **And** `authMode=oauth2` を返す
