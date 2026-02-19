# 設計: add-reply-thread-conversation

## 方針
- agentic-cli-design の原則に従い、機械可読・非対話・冪等・観測性を優先する。
- 返信/スレッド/会話取得は最小API呼び出しで構成する。

## API マッピング
- 返信投稿: `POST /2/tweets` + `reply.in_reply_to_tweet_id`
- スレッド投稿: 1件目は通常投稿、2件目以降は直前IDに返信
- 投稿取得: `GET /2/tweets/{id}`
- 会話取得: `GET /2/tweets/{id}` で `conversation_id` を特定し、
  `GET /2/tweets/search/recent?query=conversation_id:<id>` で会話投稿を収集

## 出力設計
- `tweets.reply`/`tweets.thread` は `Envelope.data.tweet` もしくは `Envelope.data.tweets[]` を返す。
- `tweets.conversation` は `posts` (フラット) と `edges` (親子) を返す。
- 失敗時は `Envelope.error` に `failedIndex`/`createdTweetIds` などの回復情報を含める。

## 冪等性
- `reply` と `thread` は `client_request_id` を受け、既存の冪等ledgerを利用する。
- `thread` は `--client-request-id-prefix` から段番号付きのIDを生成する。

## 認証・非対話
- 未認証/スコープ不足は `AuthRequired`/`AuthorizationFailed` を返す。
- `--non-interactive` ではプロンプトを禁止し、`nextSteps` を含む構造化エラーで終了する。

## テスト・モック
- 実API呼び出しは行わず、X API クライアントをインターフェース化し、
  モック実装でレスポンスを再現する。
- 会話取得は固定フィクスチャで `conversation_id` の再構成を検証する。
