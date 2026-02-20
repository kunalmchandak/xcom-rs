# tweets-replies Specification

## Purpose
TBD - created by archiving change add-reply-thread-conversation. Update Purpose after archive.
## Requirements
### Requirement: 返信投稿コマンドの提供
`xcom-rs` は `tweets reply` 実行時にX APIへ返信投稿リクエストを送信しなければならない（MUST）。

#### Scenario: 返信投稿のAPI送信
- **Given** 利用者が `tweets reply 123 "hello"` を実行する
- **When** CLIが返信投稿を作成する
- **Then** APIリクエストに `reply.in_reply_to_tweet_id=123` が含まれる
- **And** 成功時のtweet IDが `data.tweet.id` で返る

### Requirement: スレッド投稿コマンドの提供
`xcom-rs` は `tweets thread` を実行するとき、X APIへ逐次投稿を送信しなければならない（MUST）。

#### Scenario: 逐次投稿の成功
- **Given** 利用者が `tweets thread "a" "b"` を実行する
- **When** CLIがAPIへ投稿を送信する
- **Then** 2件目以降は直前tweet IDへの返信として送信される

### Requirement: 会話取得コマンドの提供
`xcom-rs` は `tweets show` と `tweets conversation` を実行するとき、X APIの結果に基づいて返さなければならず、モック実装へフォールバックしてはならない（MUST NOT）。

#### Scenario: 認証失敗時のフォールバック禁止
- **Given** APIが認証失敗を返す状態である
- **When** 利用者が `tweets conversation 123 --output json` を実行する
- **Then** `error.code=auth_required` が返る
- **And** `data.posts` は返らない

