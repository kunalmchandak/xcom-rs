## MODIFIED Requirements
### Requirement: 実行前コスト見積
`xcom-rs` は `billing estimate` で、設定されたレート表に基づいて見積を返さなければならない（MUST）。

#### Scenario: 設定レート表の適用
- **Given** レート表設定ファイルに `tweets.create=7` が定義されている
- **When** 利用者が `billing estimate tweets.create --text "hello"` を実行する
- **Then** `cost.credits` は設定値に基づく

## ADDED Requirements
### Requirement: 利用状況レポート
`xcom-rs` は `billing report` で当日の使用量を返さなければならない（MUST）。

#### Scenario: 当日使用量の返却
- **Given** 当日累積使用量が `12` クレジット
- **When** 利用者が `billing report --output json` を実行する
- **Then** `data.todayUsage=12` が返る
