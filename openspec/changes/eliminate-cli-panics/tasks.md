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
