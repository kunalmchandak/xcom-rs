# cli-core Specification Delta

## MODIFIED Requirements
### Requirement: デフォルト起動時のヘルプ提示
`xcom-rs` はサブコマンド未指定で起動された場合、CLIヘルプを標準出力に表示して成功終了しなければならない（MUST）。

#### Scenario: サブコマンドなしでの起動
- **Given** 利用者が `xcom-rs` を引数なしで実行したとき
- **When** CLIが起動したとき
- **Then** CLIヘルプが標準出力に表示される
- **And** CLIは終了コード `0` で終了する
- **And** `commands` のJSON Envelopeは出力されない
