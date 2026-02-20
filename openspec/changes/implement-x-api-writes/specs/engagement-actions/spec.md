## MODIFIED Requirements
### Requirement: いいねと取り消し
`xcom-rs` は `tweets like/unlike` 実行時にX APIへリクエストを送信しなければならない（MUST）。

#### Scenario: いいねのAPI送信
- **Given** 利用者が `tweets like 123` を実行する
- **When** CLIがAPIへ送信する
- **Then** 対象tweet IDがAPIリクエストに含まれる
- **And** 成功時は `data.tweet_id=123` が返る

### Requirement: ブックマークの追加・削除・一覧
`xcom-rs` は `bookmarks add/remove` を実行するとき、X APIへリクエストを送信しなければならない（MUST）。

#### Scenario: ブックマーク追加のAPI送信
- **Given** 利用者が `bookmarks add 123` を実行する
- **When** CLIがAPIへ送信する
- **Then** 成功時は `data.tweet_id=123` が返る
