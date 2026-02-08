# タスク

1. [x] 統合テストの実行バイナリ解決を `CARGO_BIN_EXE_xcom-rs` に置き換える
   - 対象: `tests/integration_test.rs`
   - 完了条件: ハードコードされた `./target/release/xcom-rs` の参照が削除される
   - 検証: `rg -n "target/release/xcom-rs" tests` が 0 件

2. [x] 統合テストのビルド前提を整理し、余分な `cargo build --release` を削除する
   - 対象: `tests/integration_test.rs`
   - 完了条件: テスト実行が Cargo の生成バイナリ参照に一元化される
   - 検証: `rg -n "cargo\".*build --release" tests/integration_test.rs` が 0 件

3. [x] セキュリティ監査ジョブの外部要因失敗を非致命扱いにする
   - 対象: `.github/workflows/ci.yml`
   - 完了条件: JSON 取得失敗などのツール失敗は警告として扱われる
   - 検証: `.github/workflows/ci.yml` に監査ジョブの非致命化設定がある

4. [x] 変更後のテスト実行を確認する
   - 対象: ローカル
   - 完了条件: `cargo test` が通る
   - 検証: `cargo test` を実行し成功する

## Future Work
- GitHub Actions 上での再実行結果の確認（外部システム依存のため手動）
