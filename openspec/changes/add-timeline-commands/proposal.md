# 提案: add-timeline-commands

## 目的
ホーム/メンション/ユーザーのタイムライン取得を追加し、読む機能の中核を提供する。

## 背景
CLIが投稿作成のみだと“クライアント”として成立しないため、最低限の閲覧機能が必要。

## 変更概要
- `timeline home --limit N [--cursor <token>]` を追加する。
- `timeline mentions --limit N [--cursor <token>]` を追加する。
- `timeline user <handle> --limit N [--cursor <token>]` を追加する。
- `commands`/`schema`/`help` に新コマンドを反映する。

## 非スコープ
- 高度なフィルタ（検索式/複合条件）
- リアルタイムストリーミング

## 成功条件
- 3種類のタイムラインが `--output json|ndjson` で取得できる。
- `--limit` と `--cursor` によるページングが可能。

## 依存関係・リスク
- `GET /2/users/me` と各タイムラインAPIに依存。
- 認証スコープ不足は `auth-headless` の仕様と整合させる。
