## Context

検索・タイムライン・取得系の実装がスタブのため、実データに基づく出力になっていません。共通HTTPクライアントを用いて実APIを呼び出す必要があります。

## Goals / Non-Goals

**Goals:**
- 検索・タイムライン・取得系を実API呼び出しに置き換える
- APIのページングトークン/フィールド指定をCLIレスポンスに反映する
- モックサーバで検証できるようにする

**Non-Goals:**
- 新しいCLIコマンドの追加
- 非同期化やストリーミング対応

## Decisions

- **検索 recent**: `GET /2/tweets/search/recent` を使用し、`next_token` を `meta.pagination.next_token` に反映する。
- **検索 users**: X API v1.1 の `GET /1.1/users/search.json` を使用し、`count` とページングをCLIの`limit/cursor`に合わせてマッピングする。
- **タイムライン home/mentions**: `GET /2/users/{id}/timelines/reverse_chronological` と `GET /2/users/{id}/mentions` を使用する。
- **タイムライン user**: `GET /2/users/{id}/tweets` を使用する。
- **tweets list**: 認証ユーザーの投稿を `GET /2/users/{id}/tweets` で取得し、`--fields` を `tweet.fields` にマッピングする。
- **tweets show/conversation**: `GET /2/tweets/{id}` と `GET /2/tweets/search/recent?query=conversation_id:<id>` を利用する。

## Risks / Trade-offs

- [ユーザー検索のAPI差異] → v1.1エンドポイントを使うため、必要スコープとレスポンス差異をドキュメントに明記する
- [ページングの非互換] → CLIのcursor形式とAPIのtokenを変換するレイヤを追加する
- [フィールド指定の不足] → 要求フィールド未取得時は `None` を許容し、ログで警告する
