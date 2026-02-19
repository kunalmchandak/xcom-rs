# tweets-replies Specification

## Purpose
返信・スレッド・会話取得を追加し、会話中心の操作を可能にする。

## ADDED Requirements
### Requirement: 返信投稿コマンドの提供
`xcom-rs` は `tweets reply <tweet_id> "<text>"` を提供し、指定投稿への返信を作成しなければならない（MUST）。

#### Scenario: 返信投稿の作成
- **Given** 利用者が `tweets reply 123 "hello"` を `--output json` で実行したとき
- **When** CLIが返信投稿を作成するとき
- **Then** `type="tweets.reply"` で `data.tweet.id` が返る
- **And** `reply.in_reply_to_tweet_id=123` がAPI要求に含まれる

### Requirement: スレッド投稿コマンドの提供
`xcom-rs` は `tweets thread "<t1>" "<t2>" ...` を提供し、複数投稿を連続してスレッドとして作成しなければならない（MUST）。

#### Scenario: スレッド投稿の連続作成
- **Given** 利用者が `tweets thread "a" "b" "c"` を実行したとき
- **When** CLIが投稿を作成するとき
- **Then** 2件目以降は直前投稿への返信として作成される
- **And** 失敗時は `failedIndex` と `createdTweetIds` を含む構造化エラーを返す

### Requirement: 会話取得コマンドの提供
`xcom-rs` は `tweets show <tweet_id>` と `tweets conversation <tweet_id>` を提供し、会話IDに基づく会話取得を行わなければならない（MUST）。

#### Scenario: 会話ツリーの再構成
- **Given** 利用者が `tweets conversation 123` を実行したとき
- **When** CLIが会話投稿を取得するとき
- **Then** `data.conversation_id` と `data.posts` が返る
- **And** `data.edges` に親子関係のペアが含まれる
