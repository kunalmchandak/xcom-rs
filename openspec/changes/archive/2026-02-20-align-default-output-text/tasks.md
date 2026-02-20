## 1. 早期エラーの出力形式判定

- [x] 1.1 `src/main.rs` に `--output` の簡易解析を追加し、`Cli::try_parse()` 失敗時の出力形式を決定する（確認: `xcom-rs --output json --log-format invalid commands` で JSON の `Envelope` が stdout に出る）。
- [x] 1.2 `--log-format` 検証を `output_format` 決定後に実施し、`--output` 未指定時は `text` を既定にする（確認: `xcom-rs --log-format invalid commands` の stderr が `Error:` で始まり、終了コードが `2`）。

## 2. テスト更新

- [x] 2.1 `tests/integration_test.rs` の `test_invalid_log_format` を更新し、JSON 期待のため `--output json` を明示する（確認: `cargo test test_invalid_log_format` が成功）。
- [x] 2.2 既定 `text` の早期エラーを検証するテストを追加する（確認: `cargo test early_output_default_text` などの新規テストが成功）。
- [x] 2.3 `xcom-rs auth` のサブコマンド不足が `text` で出力されることを検証するテストを追加する（確認: `cargo test early_output_missing_subcommand` などの新規テストが成功）。
- [x] 2.4 `xcom-rs auth --output txt` の不正値が text で返ることを検証するテストを追加する（確認: `cargo test early_output_invalid_output_value` などの新規テストが成功）。

## Acceptance #1 Failure Follow-up

- [x] `openspec/changes/align-default-output-text/specs/cli-core/spec.md` の「不正な出力形式の早期エラー」（JSON Envelope）と実装・テストの不整合を解消する。JSON を正とするなら `src/main.rs` の早期 `--output` 判定を修正し、`tests/integration_test.rs` の `test_invalid_output_value_fallback_to_text` を JSON 期待へ更新する。text を正とするなら spec を text 期待へ修正する。
