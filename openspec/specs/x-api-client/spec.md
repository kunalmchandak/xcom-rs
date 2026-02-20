# x-api-client Specification

## Purpose
TBD - created by archiving change introduce-x-api-client. Update Purpose after archive.
## Requirements
### Requirement: 認証付きHTTPリクエストの統一
`xcom-rs` は X API 向けに、環境変数または保存済みOAuth認証情報を解決し、BearerまたはOAuth1.0a署名のいずれかを用いてHTTPリクエストを組み立てなければならない（MUST）。

#### Scenario: OAuth1.0a 認証での送信
- **Given** OAuth1.0a の認証情報が解決される
- **When** X APIへGETリクエストを送る
- **Then** `Authorization: OAuth ...` ヘッダが付与される
- **And** `Authorization: Bearer ...` は付与されない

### Requirement: エラー分類の共通化
`xcom-rs` は X API のHTTPレスポンスを共通のエラー形式に分類しなければならない（MUST）。

#### Scenario: レート制限の分類
- **Given** X APIがHTTP 429を返す
- **When** CLIがエラーを返す
- **Then** `error.code=rate_limited` と `error.isRetryable=true` が返る
- **And** `Retry-After` または `x-rate-limit-reset` があれば `error.retryAfterMs` に反映される

### Requirement: OAuth1.0a 署名ヘッダの生成
`xcom-rs` は OAuth1.0a の認証情報でリクエストを送信する場合、HMAC-SHA1 署名を含む Authorization ヘッダを生成しなければならない（MUST）。

#### Scenario: 署名ヘッダの必須パラメータ
- **Given** OAuth1.0a の consumer key/secret と access token/secret がある
- **When** OAuth1.0a 署名ヘッダを生成する
- **Then** `oauth_consumer_key` と `oauth_signature_method=HMAC-SHA1` が含まれる

