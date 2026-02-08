# eliminate-cli-panics Tasks

1. `src/main.rs` の `LogFormat::from_str(&cli.log_format).unwrap()` を `?` に置き換え、エラー時は構造化レスポンスを返す。
   - 検証: 不正な `--log-format` 指定時に JSON エラーが返る。
2. `src/main.rs` の `.expect("Command should be present after None check")` を削除し、`if let Some(cmd) = cli.command` で処理する。
   - 検証: `cargo check` が通り、コマンド未指定時にヘルプが表示される。
3. `IdempotencyLedger::new(None).expect(...)` を `?` に置き換え、初期化失敗時は構造化エラーを返す。
   - 検証: ledger 初期化失敗時に JSON エラーが返る。
4. 既存のテストを更新し、パニックではなくエラー応答を確認する。
   - 検証: `cargo test --verbose` が成功する。
