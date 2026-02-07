## ADDED Requirements

### Requirement: 自己記述コマンド一覧
`xcom-rs` はコマンド体系を機械可読で列挙する `commands --output json` を提供しなければならない（MUST）。

#### Scenario: コマンド一覧の機械取得
- **Given** 利用者が `xcom-rs commands --output json` を実行したとき
- **When** CLIがレスポンスを返すとき
- **Then** `commands[]` にサブコマンド名、引数、説明、危険度、課金有無が含まれる

### Requirement: コマンド単位のJSON Schema公開
`xcom-rs` は `schema --command <name> --output json-schema` で入出力仕様を公開しなければならない（MUST）。

#### Scenario: 既存コマンドのSchema取得
- **Given** 利用者が `schema --command commands --output json-schema` を実行したとき
- **When** CLIがレスポンスを返すとき
- **Then** 入力引数Schemaと出力Envelope SchemaがJSON Schema形式で返る

### Requirement: ヘルプのJSON化
`xcom-rs` は `help <command> --output json` で、利用例・終了コード・エラー語彙を返さなければならない（MUST）。

#### Scenario: 失敗復旧に必要なヘルプ取得
- **Given** 利用者が `help tweets.create --output json` を実行したとき
- **When** CLIがレスポンスを返すとき
- **Then** `examples[]` `exitCodes` `errorVocabulary[]` が含まれる
