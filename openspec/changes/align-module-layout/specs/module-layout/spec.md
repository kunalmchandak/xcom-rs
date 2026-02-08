# module-layout Specification Delta

## MODIFIED Requirements
### Requirement: モジュール構成の統一
`xcom-rs` の各ドメインモジュールは一貫したファイル構成を持たなければならない（MUST）。

#### Scenario: モジュール構成の確認
- **Given** 開発者が新しいドメインモジュールを追加するとき
- **When** 既存のモジュール構成を参照するとき
- **Then** `models`, `storage`, `commands` (必要に応じて) のパターンが統一されている
- **And** モジュール間の構成が一貫している
