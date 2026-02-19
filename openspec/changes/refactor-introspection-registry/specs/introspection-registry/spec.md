# introspection-registry 仕様差分

## ADDED Requirements

### Requirement: コマンドメタデータの一貫性
`xcom-rs` は、`commands` と `help` が同一のコマンドメタデータに基づき一貫した情報を返さなければならない。

#### Scenario: 一覧とヘルプの整合性
- **Given** `commands` がコマンド一覧を返す
- **When** 同じコマンドに対して `help` を取得する
- **Then** `commands` の説明文と `help` の説明文が一致する
