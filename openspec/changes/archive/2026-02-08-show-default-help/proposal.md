# Proposal: Show help on empty invocation

## Summary
`xcom-rs` をサブコマンドなしで実行した場合、コマンドヘルプを表示して終了する挙動へ変更します。

## Why
現状はサブコマンドがない場合に `commands` 出力へフォールバックするため、利用者が期待するヘルプ表示と一致しません。
初回利用時の発見性を高め、CLIの標準的な振る舞いに揃えます。

## What Changes
- Default behavior when no subcommand is provided will show help instead of falling back to `commands` output
- Exit with status code 0 (success) when showing help on empty invocation

## Scope
- サブコマンド未指定時の表示をヘルプに変更
- 成功終了（exit code 0）

## Out of Scope
- 既存の各サブコマンドの入出力仕様変更
- JSONヘルプの構造化変更（既存の `help` コマンドは維持）

## Notes
- 既存の `commands` 取得が必要な場合は `xcom-rs commands` を明示利用する想定です。
