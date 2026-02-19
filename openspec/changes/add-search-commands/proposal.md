# 提案: add-search-commands

## 目的
投稿検索とユーザー検索を追加し、探索的な利用を可能にする。

## 背景
閲覧系機能が増えても、検索がないと目的の投稿やユーザーに到達しにくい。

## 変更概要
- `search recent "<query>" --limit N [--cursor <token>]` を追加する。
- `search users "<query>" --limit N [--cursor <token>]` を追加する。
- `commands`/`schema`/`help` に新コマンドを反映する。

## 非スコープ
- フルアーカイブ検索
- 高度なフィルタ（複数クエリ結合、複雑なexpansions）

## 成功条件
- 検索結果が `--output json|ndjson` で取得できる。
- `--limit` と `--cursor` によるページングが可能。

## 依存関係・リスク
- `GET /2/tweets/search/recent` と `GET /2/users/search` に依存。
- 取得権限は `tweet.read` / `users.read` スコープに依存。
