# x-api-client Specification (Delta)

## MODIFIED Requirements
### Requirement: 認証付きHTTPリクエストの統一
`xcom-rs` は X API 向けに、環境変数または保存済みOAuth2認証情報から解決したアクセストークンを用いてHTTPリクエストを組み立てなければならない（MUST）。

#### Scenario: 保存済みトークンでの送信
- **Given** `XCOM_RS_BEARER_TOKEN` が設定されていない
- **And** `auth.json` に有効なアクセストークンが保存されている
- **When** X APIへGETリクエストを送る
- **Then** `Authorization: Bearer <token>` が付与される
