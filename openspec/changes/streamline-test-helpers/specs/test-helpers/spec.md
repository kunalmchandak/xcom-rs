# test-helpers Specification Delta

## MODIFIED Requirements
### Requirement: テストヘルパーの提供
`xcom-rs` はテストコードの可読性を高めるため、共通のテストヘルパーを提供しなければならない（MUST）。

#### Scenario: テストヘルパーの使用
- **Given** 開発者がテストを書くとき
- **When** 共通のセットアップが必要なとき
- **Then** テストヘルパー関数を使用できる
- **And** テストコードの重複が削減される
