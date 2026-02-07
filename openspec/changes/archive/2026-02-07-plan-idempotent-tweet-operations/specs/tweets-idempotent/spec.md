## ADDED Requirements

### Requirement: 投稿作成の冪等実行
`xcom-rs` は `tweets create` で `--client-request-id` を受け付け、同一IDの再実行に対して冪等に振る舞わなければならない（MUST）。

#### Scenario: 同一ID再実行で重複投稿を防止
- **Given** 利用者が `tweets create --client-request-id req-1 --text "hello"` を実行し成功した
- **When** 同じ `client-request-id` と同じペイロードで再実行する
- **Then** CLIは新規投稿を作らず、既存の `tweetId` を返す

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
