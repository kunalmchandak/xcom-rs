# 提案: add-reply-thread-conversation

## 目的
`xcom-rs` に返信・スレッド投稿・会話取得の最小機能を追加し、会話中心の利用を成立させる。

## 背景
現状は `tweets create/list` のみで、返信や会話取得ができないため、実運用での対話が成立しない。

## 変更概要
- `tweets reply <tweet_id> "<text>"` を追加し、返信投稿を作成する。
- `tweets thread "<t1>" "<t2>" ...` を追加し、複数投稿をスレッドとして連続投稿する。
- `tweets show <tweet_id>` を追加し、投稿詳細を取得する。
- `tweets conversation <tweet_id>` を追加し、`conversation_id` を起点に会話ツリーを取得・再構成する。
- すべての新コマンドを `commands`/`schema`/`help` に反映する。

## 非スコープ
- UI/インタラクティブ操作の追加
- ストリーミング購読やリアルタイム監視
- サーバ側での会話ツリー構築（APIに依存）

## 成功条件
- 返信/スレッド/会話取得が機械可読なレスポンスで完了する。
- 非対話モードでの認証不足が構造化エラーで返る。
- 会話取得が `conversation_id` による検索で再構成できる。

## 依存関係・リスク
- X API の `POST /2/tweets` と `GET /2/tweets/{id}`、`GET /2/tweets/search/recent` に依存。
- 認証スコープ不足時のエラー処理は既存の `auth-headless` 仕様と整合させる。
