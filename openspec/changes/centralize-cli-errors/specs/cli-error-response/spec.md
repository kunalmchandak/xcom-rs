# cli-error-response Specification Delta

## MODIFIED Requirements
### Requirement: 失敗レスポンスの統一生成
`xcom-rs` は失敗レスポンスの `Envelope` と `ErrorDetails` を共通の生成経路で構築しなければならない（MUST）。

#### Scenario: 共通ヘルパーの使用
- **Given** いずれかの CLI コマンドが失敗したとき
- **When** CLI がエラー応答を生成するとき
- **Then** 失敗レスポンスは共通ヘルパー経由で構築される
- **And** 既存の `error.code` `error.message` `error.isRetryable` の形式は維持される
