## 1. CLI 追加とルーティング

- [x] 1.1 `tweets like/unlike/retweet/unretweet` と `bookmarks add/remove/list` を定義する（確認: `src/cli.rs` に引数定義が追加されている）
- [x] 1.2 追加コマンドをハンドラに配線する（確認: `src/handlers/` で分岐が追加されている）

## 2. エンゲージ処理ロジック

- [x] 2.1 いいね/リツイート/解除の処理を実装する（確認: `src/tweets/commands.rs` に対応関数が存在する）
- [x] 2.2 ブックマーク追加/削除/一覧の処理を実装する（確認: `src/bookmarks/commands.rs` に対応関数が存在する）

## 3. API クライアントとモック

- [x] 3.1 X API 呼び出しのためのクライアントIFとモック実装を追加する（確認: `src/test_utils.rs` の `engagement_fixtures` モジュールにモックデータが実装されている）
- [x] 3.2 ブックマーク一覧のページングを再現するフィクスチャを追加する（確認: テスト用データが `src/test_utils.rs` の `engagement_fixtures` モジュールに追加されている）

## 4. イントロスペクションとコスト

- [x] 4.1 `commands/schema/help` に新コマンドを追加する（確認: `src/introspection.rs` の一覧とSchemaが更新されている）
- [x] 4.2 コスト見積の operation key を追加する（確認: `src/billing/storage.rs` に `tweets.like` 等が追加されている）

## 5. テスト

- [x] 5.1 エンゲージ操作のユニットテストを追加する（確認: `src/tweets/commands.rs` と `src/bookmarks/commands.rs` の `#[cfg(test)]` にテストがある）
- [x] 5.2 CLI パーステストを追加する（確認: `src/cli.rs` のテストに新サブコマンドが含まれる）

## Acceptance #1 Failure Follow-up

- [x] Fix flaky env-var interference in `src/tweets/commands.rs::tests::test_list_with_field_projection` by using `crate::test_utils::env_lock::ENV_LOCK` and clearing `XCOM_SIMULATE_ERROR`/`XCOM_RETRY_AFTER_MS` before assertions; re-run `cargo test --verbose` to confirm no regression.

## Acceptance #2 Failure Follow-up

- [x] Fix doctest regression so `cargo test --verbose` passes end-to-end: current run fails in doc-tests with `E0463` (`can't find crate for tracing_subscriber` at `src/logging.rs:2` and `can't find crate for rusqlite` at `src/tweets/ledger.rs:2`). Ensure doc-test linkage/dependency resolution is stable, then re-run `cargo test --verbose`.
