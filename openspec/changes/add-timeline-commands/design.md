# 設計: add-timeline-commands

## 方針
- `timeline` は読み取り専用で、`--limit`/`--cursor` によるページングを基本とする。
- 認証ユーザーIDは `GET /2/users/me` で解決し、必要ならキャッシュする。

## API マッピング
- home: `GET /2/users/{id}/timelines/reverse_chronological`
- mentions: `GET /2/users/{id}/mentions`
- user: `GET /2/users/by/username/{handle}` → `GET /2/users/{id}/tweets`

## 出力設計
- `timeline.home` / `timeline.mentions` / `timeline.user` の `data.tweets[]` に投稿配列を返す。
- ページング情報は `data.meta.pagination` に `next_token`/`previous_token` を格納する。
- `--output ndjson` では `tweets[]` を1行ずつ出力する。

## 認証・非対話
- 未認証時は `AuthRequired` を返し、`nextSteps` で復旧手順を示す。

## テスト・モック
- X API クライアントのモックで `timeline` レスポンスを固定化する。
