# cli-core Specification

## Purpose
TBD - created by archiving change design-agentic-cli-core. Update Purpose after archive.
## Requirements
### Requirement: Common Response Envelope
`xcom-rs` の全コマンドは、機械可読な共通レスポンスEnvelopeを返さなければならない（MUST）。
Envelopeは `ok` `type` `schemaVersion` をトップレベル固定とし、成功時は `data`、失敗時は `error` を含む。

#### Scenario: 成功レスポンスの固定キー
- **Given** 利用者が任意の成功コマンドを `--output json` で実行したとき
- **When** CLIが結果を返すとき
- **Then** `ok=true` `type` `schemaVersion=1` `data` が存在する
- **And** `stdout` には結果JSONのみが出力される

#### Scenario: 失敗レスポンスの構造化
- **Given** 利用者が不正な引数でコマンドを実行したとき
- **When** CLIが失敗を返すとき
- **Then** `ok=false` `type` `schemaVersion=1` `error.code` `error.message` `error.isRetryable` が存在する
- **And** CLIは終了コード `2` を返す

### Requirement: Non-interactive Default and Observability
`xcom-rs` は非対話環境で停止しないように設計され、ログ相関情報を出力できなければならない（MUST）。

#### Scenario: 非対話モードでの実行完了
- **Given** 利用者が `--non-interactive` を付けてコマンドを実行したとき
- **When** 追加入力が必要な状況が発生したとき
- **Then** CLIは対話プロンプトを表示せず、構造化エラーと次の手順を返す

#### Scenario: trace-idの伝播
- **Given** 利用者が `--trace-id abc-123` を指定して実行したとき
- **When** CLIが結果を返すとき
- **Then** 結果の `meta.traceId` とstderrログの相関IDが `abc-123` になる

### Requirement: Show Help on Default Launch
`xcom-rs` MUST display CLI help to standard output and exit successfully when launched without a subcommand.

#### Scenario: Launch without subcommand
- **Given** a user executes `xcom-rs` without arguments
- **When** the CLI launches
- **Then** CLI help is displayed to standard output
- **And** the CLI exits with status code `0`
- **And** the `commands` JSON Envelope is not output

### Requirement: Early Error Output Format Alignment
`xcom-rs` MUST return early errors (such as argument parsing failures or invalid `--log-format` values) using `text` as the default output format when `--output` is not specified. When `--output` is explicitly provided, early errors MUST respect the specified format.

#### Scenario: Early error with default output
- **Given** user runs `xcom-rs --log-format invalid commands` without specifying `--output`
- **When** CLI returns an error
- **Then** error is output in `text` format
- **And** CLI returns exit code `2`

#### Scenario: Early error without subcommand
- **Given** user runs `xcom-rs auth` without specifying `--output`
- **When** CLI returns an error
- **Then** error is output in `text` format
- **And** CLI returns exit code `2`

#### Scenario: Early error with explicit output format
- **Given** user runs `xcom-rs --output json --log-format invalid commands`
- **When** CLI returns an error
- **Then** error is output in JSON `Envelope` format
- **And** CLI returns exit code `2`

#### Scenario: Early error with invalid output format
- **Given** user runs `xcom-rs auth --output txt`
- **When** CLI returns an error
- **Then** invalid `--output` value falls back to default `text` and error is output in `text` format
- **And** CLI returns exit code `2`

