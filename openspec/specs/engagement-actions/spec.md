# engagement-actions Specification

## Purpose
TBD - created by archiving change add-engagement-actions. Update Purpose after archive.
## Requirements
### Requirement: いいねと取り消し
`xcom-rs` は `tweets like <tweet_id>` と `tweets unlike <tweet_id>` を提供しなければならない（MUST）。

#### Scenario: いいねの作成
- **Given** 利用者が `tweets like 123` を `--output json` で実行したとき
- **When** CLIがいいねを作成するとき
- **Then** `type="tweets.like"` で `data.tweet_id=123` が返る

### Requirement: リツイートと解除
`xcom-rs` は `tweets retweet <tweet_id>` と `tweets unretweet <tweet_id>` を提供しなければならない（MUST）。

#### Scenario: リツイートの解除
- **Given** 利用者が `tweets unretweet 123` を実行したとき
- **When** CLIが解除を実行するとき
- **Then** `type="tweets.unretweet"` が返る

### Requirement: ブックマークの追加・削除・一覧
`xcom-rs` は `bookmarks add/remove/list` を提供し、ブックマーク操作を行わなければならない（MUST）。

#### Scenario: ブックマーク一覧の取得
- **Given** 利用者が `bookmarks list --limit 20` を実行したとき
- **When** CLIが一覧を返すとき
- **Then** `data.tweets` が最大20件で返る
- **And** 続きがある場合 `data.meta.pagination.next_token` が返る

