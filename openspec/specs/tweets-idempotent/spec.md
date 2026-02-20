# tweets-idempotent Specification

## Purpose
TBD - created by archiving change plan-idempotent-tweet-operations. Update Purpose after archive.
## Requirements
### Requirement: 投稿作成の冪等実行
`xcom-rs` は `tweets create` の実行時、スタブではなくX APIへリクエストを送信し、成功時のtweet IDを冪等結果として保存しなければならない（MUST）。

#### Scenario: 実API呼び出しの結果保存
- **Given** 利用者が `tweets create --text "hello"` を実行する
- **When** X APIが作成済みtweet IDを返す
- **Then** CLIはそのIDをレスポンスに含める
- **And** 同一 `client-request-id` 再実行時は保存済みIDを返す

### Requirement: 既存時の挙動選択
`xcom-rs` は同一ID既存時の動作を `--if-exists return|error` で指定できなければならない（MUST）。

#### Scenario: errorモードでの失敗
- **Given** 同一 `client-request-id` が既に存在する
- **And** 利用者が `--if-exists error` を指定する
- **When** `tweets create` を実行する
- **Then** `error.code=idempotency_conflict` を返し終了する

### Requirement: 再試行可能性の明示
`xcom-rs` は投稿系エラーに `error.isRetryable` と必要に応じた待機情報を含めなければならない（MUST）。

#### Scenario: レート制限時の再試行情報
- **Given** X APIがレート制限応答を返す
- **When** CLIがエラーを返す
- **Then** `error.code=rate_limited` `error.isRetryable=true` `error.retryAfterMs` を返す
- **And** 終了コード `4` を返す

