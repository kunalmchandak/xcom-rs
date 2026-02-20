# x-api-client Specification

## Purpose
TBD - created by archiving change introduce-x-api-client. Update Purpose after archive.
## Requirements
### Requirement: 認証付きHTTPリクエストの統一
`xcom-rs` は X API 向けに、ベースURLと認証ヘッダーを統一したHTTPリクエストを組み立てなければならない（MUST）。

#### Scenario: Bearerトークン付きの送信
- **Given** AuthStore に `XCOM_RS_BEARER_TOKEN` が設定されている
- **When** X APIへGETリクエストを送る
- **Then** `Authorization: Bearer <token>` が付与される
- **And** ベースURLが設定値（`XCOM_RS_API_BASE`）に従う

### Requirement: エラー分類の共通化
`xcom-rs` は X API のHTTPレスポンスを共通のエラー形式に分類しなければならない（MUST）。

#### Scenario: レート制限の分類
- **Given** X APIがHTTP 429を返す
- **When** CLIがエラーを返す
- **Then** `error.code=rate_limited` と `error.isRetryable=true` が返る
- **And** `Retry-After` または `x-rate-limit-reset` があれば `error.retryAfterMs` に反映される

