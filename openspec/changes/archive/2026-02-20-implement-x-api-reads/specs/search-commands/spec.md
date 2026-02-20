## MODIFIED Requirements
### Requirement: 最近の投稿検索
`xcom-rs` は `search recent` 実行時にX APIの検索エンドポイントから取得しなければならない（MUST）。

#### Scenario: 実API検索のページング
- **Given** 利用者が `search recent "from:OpenAI" --limit 10` を実行する
- **When** CLIがAPIから結果を取得する
- **Then** `data.tweets` はAPIの結果に基づく
- **And** 続きがある場合 `data.meta.pagination.next_token` が返る

### Requirement: ユーザー検索
`xcom-rs` は `search users` 実行時にX APIのユーザー検索から取得しなければならない（MUST）。

#### Scenario: 実APIユーザー検索
- **Given** 利用者が `search users "xdev" --limit 5` を実行する
- **When** CLIがAPIから結果を取得する
- **Then** `type="search.users"` のレスポンスが返る
