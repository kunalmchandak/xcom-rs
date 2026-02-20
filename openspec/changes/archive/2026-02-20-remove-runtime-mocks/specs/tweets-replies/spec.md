## MODIFIED Requirements

### Requirement: 会話取得コマンドの提供
`xcom-rs` は `tweets show` と `tweets conversation` を実行するとき、X APIの結果に基づいて返さなければならず、モック実装へフォールバックしてはならない（MUST NOT）。

#### Scenario: 認証失敗時のフォールバック禁止
- **Given** APIが認証失敗を返す状態である
- **When** 利用者が `tweets conversation 123 --output json` を実行する
- **Then** `error.code=auth_required` が返る
- **And** `data.posts` は返らない
