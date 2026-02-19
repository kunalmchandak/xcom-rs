# cli-parse-tests 仕様差分

## ADDED Requirements

### Requirement: CLI パースの回帰防止
CLI パース結果はテーブル駆動テストで網羅され、主要コマンドの解釈が変わらないことを保証しなければならない。

#### Scenario: パース結果の一致
- **Given** 既存の CLI 引数（例: `tweets create`, `bookmarks list`）
- **When** パースを実行する
- **Then** 期待されるコマンド種別と引数値が一致する
