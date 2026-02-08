# cli-robustness Specification Delta

## ADDED Requirements
### Requirement: 本番コードのパニック排除
`xcom-rs` の本番実行パスは `unwrap` / `expect` によるパニックを起こしてはならない（MUST NOT）。

#### Scenario: 初期化失敗時の構造化エラー
- **Given** CLI の初期化処理が失敗したとき
- **When** CLI がエラーを返すとき
- **Then** パニックせず、`Envelope` 形式のエラーが出力される
- **And** 適切な終了コードが返される
