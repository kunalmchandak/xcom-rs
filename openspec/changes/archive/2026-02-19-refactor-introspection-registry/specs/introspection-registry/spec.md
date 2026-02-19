# introspection-registry 仕様差分

## ADDED Requirements

### Requirement: コマンドメタデータの一貫性
`xcom-rs` MUST return consistent information from `commands` and `help` based on the same command metadata source.

#### Scenario: 一覧とヘルプの整合性
- **Given** `commands` がコマンド一覧を返す
- **When** 同じコマンドに対して `help` を取得する
- **Then** `commands` の説明文と `help` の説明文が一致する
