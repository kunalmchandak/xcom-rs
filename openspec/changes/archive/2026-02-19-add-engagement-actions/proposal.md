# 提案: add-engagement-actions

## 目的
いいね/リツイート/ブックマークの操作を追加し、閲覧後の基本的なエンゲージメントを可能にする。

## 背景
現在のCLIは投稿作成と閲覧が中心で、反応操作が不足しているため“読む→反応”の最小サイクルが成立しない。

## 変更概要
- `tweets like <tweet_id>` を追加する。
- `tweets unlike <tweet_id>` を追加する。
- `tweets retweet <tweet_id>` を追加する。
- `tweets unretweet <tweet_id>` を追加する。
- `bookmarks add <tweet_id>` を追加する。
- `bookmarks remove <tweet_id>` を追加する。
- `bookmarks list --limit N [--cursor <token>]` を追加する。
- `commands`/`schema`/`help` に新コマンドを反映する。

## 非スコープ
- 引用投稿（`quote`）や返信作成
- ブックマークのフォルダ管理
- 高度なフィルタ条件

## 成功条件
- 主要なエンゲージ操作が `--output json|ndjson` で実行できる。
- `bookmarks list` が `--limit` と `--cursor` によるページングを提供する。

## 依存関係・リスク
- `GET /2/users/me` とエンゲージ系APIに依存。
- スコープ不足は `auth-headless` と整合したエラーを返す必要がある。
