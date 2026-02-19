# 設計: add-engagement-actions

## 方針
- すべての書き込み操作は非対話で失敗し、`AuthRequired` を返す。
- 操作対象ユーザーは `GET /2/users/me` で取得し、可能ならキャッシュする。

## API マッピング
- like: `POST /2/users/{id}/likes`
- unlike: `DELETE /2/users/{id}/likes/{tweet_id}`
- retweet: `POST /2/users/{id}/retweets`
- unretweet: `DELETE /2/users/{id}/retweets/{source_tweet_id}`
- bookmarks list: `GET /2/users/{id}/bookmarks`
- bookmarks add: `POST /2/users/{id}/bookmarks`
- bookmarks remove: `DELETE /2/users/{id}/bookmarks/{tweet_id}`

## 出力設計
- `type` は `tweets.like` / `tweets.unlike` / `tweets.retweet` / `tweets.unretweet` / `bookmarks.add` / `bookmarks.remove` / `bookmarks.list` を使用する。
- 書き込み系は `data.tweet_id` と `data.success` を返す。
- 一覧は `data.tweets[]` と `data.meta.pagination.next_token` を返す。

## イントロスペクションとコスト
- `commands/schema/help` に新コマンドを追加する。
- コスト見積の operation key に `tweets.like` などを追加する。

## テスト・モック
- X API クライアントのモックで like/retweet/bookmarks のレスポンスを固定化する。
