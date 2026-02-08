# eliminate-cli-panics Tasks

- [x] 1. `src/main.rs` の `LogFormat::from_str(&cli.log_format).unwrap()` をエラーハンドリングに置き換え、エラー時は構造化レスポンスを返す。
   - 検証: 不正な `--log-format` 指定時に JSON エラーが返る。
   - 実装: `match` でエラーハンドリングし、`ErrorDetails` と `Envelope` で構造化エラーを返すように修正。
- [x] 2. `src/main.rs` の `.expect("Command should be present after None check")` を削除し、`match` パターンで処理する。
   - 検証: `cargo check` が通り、コマンド未指定時にヘルプが表示される。
   - 実装: `cli.command` を `match` で処理し、`None` の場合はヘルプを表示して正常終了するように修正。
- [x] 3. `src/handlers/tweets.rs` の `IdempotencyLedger::new(None).expect(...)` を `?` に置き換え、初期化失敗時は構造化エラーを返す。
   - 検証: ledger 初期化失敗時に JSON エラーが返る。
   - 実装: `.map_err()` でエラーメッセージを追加し、`?` で伝播するように修正。
- [x] 4. 既存のテストを確認し、パニックではなくエラー応答を確認する。
   - 検証: `cargo test --verbose` が成功する。
   - 確認結果: テストコード内の `unwrap`/`expect` は非スコープのため、更新不要。全テストが成功。
- [x] 5. コード品質チェック (`make check`: fmt, clippy, test) を実行し、全て成功することを確認。
   - 検証結果: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --verbose` 全て成功。

## Acceptance #1 Failure Follow-up

- [x] `src/logging.rs` の `LogFormat::from_str` を失敗可能な実装に変更し、未知の `--log-format` 値で `Err` を返すようにする（現状は `type Err = Infallible` かつ未知値を `Text` にフォールバックしており、`src/main.rs` のエラーハンドリング分岐が実行不能）。
   - 実装: `type Err = String` に変更し、`json`/`text` 以外の値で `Err` を返すように修正。テストも更新。
- [x] `src/main.rs` の `--log-format` エラーパスが実際に通ることを CLI テストで検証し、無効値指定時に `Envelope` 形式エラーと適切な終了コードを返すことを確認する。
   - 実装: `tests/integration_test.rs` に `test_invalid_log_format`, `test_valid_log_format_json`, `test_valid_log_format_text` を追加。無効値で exit code 2 と JSON エラーが返ることを確認。
- [x] 本番実行パスの `unwrap` を排除するため、`src/auth.rs` の `AuthStore::status`（`duration_since(...).unwrap()`）と `src/billing.rs` の `BudgetTracker::today`（`duration_since(...).unwrap()`）をエラーハンドリングへ置き換える。
   - 実装: `src/auth.rs` の `AuthStore::status` で `duration_since` のエラーを `match` でハンドリングし、システム時刻エラー時はその旨のメッセージを返すように修正。`src/billing.rs` の `BudgetTracker::today` も同様に修正。
