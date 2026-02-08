# execution-policy Specification Delta

## MODIFIED Requirements
### Requirement: 実行ポリシーの分離
`xcom-rs` は実行時のバリデーションロジックを `ExecutionContext` から分離しなければならない（MUST）。

#### Scenario: バリデーションの独立実行
- **Given** 開発者が実行ポリシーをテストするとき
- **When** `ExecutionPolicy` を単独でインスタンス化するとき
- **Then** `ExecutionContext` に依存せずバリデーションロジックをテストできる
- **And** 既存の挙動は維持される
