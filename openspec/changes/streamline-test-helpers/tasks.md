# streamline-test-helpers Tasks

1. `src/test_utils.rs` にテスト用ヘルパー関数を追加する（例: `create_test_ledger()`, `create_test_auth_store()`）。
   - 検証: ヘルパー関数が追加され、`cargo check` が通る。
2. テストコード内の繰り返しパターンをヘルパー呼び出しに置き換える。
   - 検証: テストコードの行数が削減される。
3. 環境変数依存のテストを明示的にマークする（例: `#[cfg_attr(not(feature = "env-tests"), ignore)]`）。
   - 検証: `cargo test` で環境変数依存テストがスキップされる。
4. 既存のテストが成功することを確認する。
   - 検証: `cargo test --verbose` が成功する。
