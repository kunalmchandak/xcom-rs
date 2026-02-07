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
