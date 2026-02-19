# 設計: add-search-commands

## 方針
- クエリはX APIの検索構文にそのまま渡す。
- `--limit`/`--cursor` のみを標準オプションとする。

## API マッピング
- recent: `GET /2/tweets/search/recent`
- users: `GET /2/users/search`

## 出力設計
- `type` は `search.recent` / `search.users` を使用する。
- `search.recent` は `data.tweets[]` を返す。
- `search.users` は `data.users[]` を返す。
- ページングは `data.meta.pagination.next_token` に統一する。

## イントロスペクションとコスト
- `commands/schema/help` に新コマンドを追加する。
- コスト見積の operation key に `search.recent` / `search.users` を追加する。

## テスト・モック
- X API クライアントのモックで検索レスポンスを固定化する。
