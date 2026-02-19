# cli-completions 仕様変更

## ADDED Requirements
### Requirement: 補完スクリプトを生成するコマンドを提供する
`xcom-rs` は `completion --shell <bash|zsh|fish>` で補完スクリプトを生成しなければならない（MUST）。

#### Scenario: bash補完の生成
Given 利用者が `xcom-rs completion --shell bash` を実行する
When CLIがレスポンスを返す
Then bash用補完スクリプトが標準出力に出力される

#### Scenario: zsh/fish補完の生成
Given 利用者が `xcom-rs completion --shell zsh` または `--shell fish` を実行する
When CLIがレスポンスを返す
Then 対応するシェルの補完スクリプトが標準出力に出力される

### Requirement: 補完導入手順をドキュメントに記載する
READMEとdocs/examplesに補完の導入手順を記載しなければならない（MUST）。

#### Scenario: ドキュメントに導入手順がある
Given 利用者がREADMEまたはdocs/examplesを参照する
When 補完の導入方法を探す
Then `xcom-rs completion --shell <...>` を用いた手順が記載されている
