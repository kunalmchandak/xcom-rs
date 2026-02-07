- [x] `tweets create` に `--client-request-id` と `--if-exists return|error` を追加する（確認: `--output json` で `meta.clientRequestId` が返る）。
- [x] client-request-id未指定時の自動採番を実装する（確認: 連続実行で異なるUUIDが発行される）。
- [x] idempotency台帳（sqliteまたは同等）を実装し、`client_request_id -> tweet_id` を保存する（確認: 1回目成功後、同一ID再実行で外部呼び出しなしに同一tweetIdが返る）。
- [x] タイムアウト後の再実行フローを実装する（確認: mockで初回タイムアウト、再実行で既存結果返却が成功する）。
- [x] リトライ可能エラー（429/5xx）と非リトライエラー（4xx）を分類する（確認: `error.isRetryable` がステータス別に正しい）。
- [x] `tweets list` に `--fields` `--limit` `--cursor` を実装する（確認: `--fields id,text` 指定で不要フィールドが出ない）。
- [x] `--output ndjson` を実装し、大量結果を1行1JSONで出力する（確認: 複数件取得時に行単位JSONになる）。
- [x] 外部依存を避けるため、tweets操作のHTTPモック/fixtureテストを追加する（確認: APIキー未設定でもCIテストが成功する）。

## Acceptance #1 Failure Follow-up

- [x] `tweets create --if-exists error` の重複時に `error.code=idempotency_conflict` を返すように実装する（現状は `src/main.rs` の `Commands::Tweets::Create` で `INTERNAL_ERROR` に変換される）。
- [x] `client_request_id` を単独キーとして重複判定するように台帳設計とcreateフローを修正する（現状 `src/tweets/ledger.rs` のPRIMARY KEYが `(client_request_id, request_hash)` のため、同一IDで別ペイロードを新規作成できてしまう）。
- [x] 投稿系エラーで `error.isRetryable` と必要時 `error.retryAfterMs` を実際のCLIエラーフローで返すように統合する（`src/tweets/commands.rs` の `ClassifiedError` は現状テスト内でしか使われていない）。
- [x] `tweets list --limit --cursor` の結果に `meta.pagination` を含め、`--cursor` 入力を実処理に反映する（現状 `src/tweets/commands.rs::TweetCommand::list` は `args.cursor` 未使用で `data.next_cursor` のみ返す）。

## Acceptance #2 Failure Follow-up

- [x] 同一 `client_request_id` の既存判定をペイロード差分に依存させないよう修正する（`src/tweets/commands.rs::TweetCommand::create` の `ledger.lookup(&client_request_id, &request_hash)` と `src/tweets/ledger.rs::IdempotencyLedger::lookup` のハッシュ一致フィルタにより、`client_request_id` が既存でも別テキストで新規投稿される）。
- [x] `tweets create --if-exists error` のエラーコードを仕様値 `idempotency_conflict` に一致させる（現状は `src/protocol.rs::ErrorCode` の `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]` により `IDEMPOTENCY_CONFLICT` が返る）。
- [x] 投稿系の再試行分類を実フローで発火可能に統合し、仕様どおり `error.code=rate_limited` と `error.retryAfterMs` を返せるようにする（`ClassifiedError::from_status_code/timeout` の生成が `src/tweets/commands.rs` のテスト内のみで、CLI経路では実際に生成されない）。

## Acceptance #3 Failure Follow-up

- [x] 投稿系で `ClassifiedError` を本番フローから実際に生成・伝播するよう統合する（現状 `ClassifiedError::from_status_code`/`timeout` の呼び出しは `src/tweets/commands.rs` と `tests/tweets_integration_test.rs` のテスト内のみで、`src/tweets/commands.rs::TweetCommand::create`/`list` からは生成されない）。
- [x] レート制限時の `error.code` を仕様値 `rate_limited` に一致させる（現状 `src/tweets/commands.rs::ClassifiedError::to_error_code` は `ErrorCode::RateLimitExceeded` を返し、`src/protocol.rs` の `snake_case` シリアライズで `rate_limit_exceeded` になる）。
- [x] レート制限時の待機情報を `error.retryAfterMs` 直下で返すようレスポンススキーマを修正する（現状 `src/main.rs` の `TweetsCommands::Create`/`List` のエラーハンドリングは `retryAfterMs` を `error.details.retryAfterMs` にのみ格納している）。
