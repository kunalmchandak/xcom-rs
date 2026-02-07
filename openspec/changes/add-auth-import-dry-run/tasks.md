- [x] 1. `auth import --dry-run` フラグを追加する
   - 変更箇所: `src/cli.rs`
   - 検証: `xcom-rs auth import --help` に `--dry-run` が表示される
- [x] 2. 変更計画モデルを実装する
   - 変更箇所: `src/auth.rs` または新規モジュール
   - 検証: 計画が create/update/skip/fail に分類される
- [x] 3. dry-run時の保存抑止を実装する
   - 変更箇所: `src/auth.rs`
   - 検証: テストで保存ファイルの更新が起きないことを確認
- [x] 4. 計画のJSON出力をEnvelopeに統合する
   - 変更箇所: `src/main.rs`
   - 検証: `--output json` で計画が `data` に出る
- [x] 5. テストを追加する
   - 変更箇所: `tests/`
   - 検証: `cargo test auth_import_dry_run` がパスする

## Acceptance #1 Failure Follow-up

- [x] `src/auth.rs` の `AuthStore::import_with_plan` は既存トークンがある場合に常に `ImportAction::Update` を返しており（`src/auth.rs:248`）、同一データ再インポート時の `ImportAction::Skip` 分類が実装されていません。既存トークンと入力トークンを比較して no-op 時に `ImportPlan::skip(...)` を返す実装を追加し、`tests/auth_billing_test.rs` に skip ケース（同一データ dry-run で `data.action == "skip"`）の統合テストを追加してください。
