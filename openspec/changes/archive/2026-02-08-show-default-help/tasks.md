# Tasks

- [x] サブコマンド未指定時の分岐を追加し、ヘルプ出力と成功終了に切り替える
  - 対象: `src/main.rs`
  - 検証: `cargo run --` 実行時にヘルプが表示され、終了コードが0であることを確認

- [x] 既存の `commands` フォールバックが実行されなくなることを確認する
  - 対象: `src/main.rs`
  - 検証: `cargo run --` 実行時に `commands` のJSON Envelopeが出出力されないことを確認

- [x] 仕様変更の整合性を検証する
  - 対象: `openspec/changes/show-default-help/specs/cli-core/spec.md`
  - 検証: `npx @fission-ai/openspec@latest validate show-default-help --strict` が成功することを確認
