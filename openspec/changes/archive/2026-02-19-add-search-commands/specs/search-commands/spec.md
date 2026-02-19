# search-commands Specification

## Purpose
投稿検索とユーザー検索を追加する。

## ADDED Requirements
### Requirement: 最近の投稿検索
`xcom-rs` は `search recent "<query>"` を提供し、投稿検索を実行しなければならない（MUST）。

#### Scenario: recent 検索のページング
- **Given** 利用者が `search recent "from:OpenAI" --limit 10` を実行したとき
- **When** CLIが結果を返すとき
- **Then** `data.tweets` が最大10件で返る
- **And** 続きがある場合 `data.meta.pagination.next_token` が返る

### Requirement: ユーザー検索
`xcom-rs` は `search users "<query>"` を提供し、ユーザー検索を実行しなければならない（MUST）。

#### Scenario: users 検索の取得
- **Given** 利用者が `search users "xdev" --limit 5` を実行したとき
- **When** CLIが結果を返すとき
- **Then** `type="search.users"` のレスポンスが返る
