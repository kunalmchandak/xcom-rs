# tweets-command-layout 仕様差分

## ADDED Requirements

### Requirement: `tweets` コマンドの構成整理
`tweets` コマンド群の内部実装は、機能単位のモジュールに分割されていなければならない。

#### Scenario: 回帰防止の確認
- **Given** 既存の `tweets create` 実行
- **When** 参照実装とリファクタ後の実装を比較する
- **Then** 返却される JSON の構造と主要フィールドが一致する
