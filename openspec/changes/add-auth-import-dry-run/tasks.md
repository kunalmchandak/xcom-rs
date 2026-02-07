1. `auth import --dry-run` フラグを追加する
   - 変更箇所: `src/cli.rs`
   - 検証: `xcom-rs auth import --help` に `--dry-run` が表示される
2. 変更計画モデルを実装する
   - 変更箇所: `src/auth.rs` または新規モジュール
   - 検証: 計画が create/update/skip/fail に分類される
3. dry-run時の保存抑止を実装する
   - 変更箇所: `src/auth.rs`
   - 検証: テストで保存ファイルの更新が起きないことを確認
4. 計画のJSON出力をEnvelopeに統合する
   - 変更箇所: `src/main.rs`
   - 検証: `--output json` で計画が `data` に出る
5. テストを追加する
   - 変更箇所: `tests/`
   - 検証: `cargo test auth_import_dry_run` がパスする
