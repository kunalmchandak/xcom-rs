## MODIFIED Requirements
### Requirement: 会話取得コマンドの提供
`xcom-rs` は `tweets show` と `tweets conversation` を実行するとき、X APIから取得しなければならない（MUST）。

#### Scenario: 会話ツリーのAPI取得
- **Given** 利用者が `tweets conversation 123` を実行する
- **When** CLIがAPIから会話投稿を取得する
- **Then** `data.conversation_id` と `data.posts` がAPI結果に基づく
