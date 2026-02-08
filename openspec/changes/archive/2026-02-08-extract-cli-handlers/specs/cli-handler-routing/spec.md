# cli-handler-routing Specification Delta

## ADDED Requirements
### Requirement: コマンド実行の分離
`xcom-rs` は CLI コマンドの実行をドメイン別ハンドラーに委譲し、`main.rs` はルーティングと共通初期化のみを担当しなければならない（MUST）。

#### Scenario: main.rs の薄いルーティング
- **Given** 開発者が `src/main.rs` を確認するとき
- **When** コマンドの実行経路を追うとき
- **Then** `main.rs` には `handlers::*` への委譲のみが存在する
- **And** コマンド固有のビジネスロジックは各ハンドラーに集約されている
