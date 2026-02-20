## Why

書き込み系のコマンド（投稿作成・返信・スレッド・いいね/RT・ブックマーク・メディアアップロード）がスタブのため、実運用でAPI連携できません。共通クライアントを使って実API呼び出しを行えるようにします。

## What Changes

- `tweets create/reply/thread` を実API呼び出しに置き換える
- `tweets like/unlike/retweet/unretweet` と `bookmarks add/remove/list` を実API呼び出しに置き換える
- `media upload` を実API呼び出しに置き換える
- 429/401/403/5xx のエラー分類を共通クライアント経由で統一する

## Capabilities

### New Capabilities
- なし

### Modified Capabilities
- `tweets-idempotent`: 実API呼び出し時のエラー分類と結果反映を追加
- `tweets-replies`: 返信/スレッド作成を実API化
- `engagement-actions`: いいね/RT/ブックマークを実API化
- `media-upload`: アップロードを実API化

## Impact

- `src/tweets/commands/*` と `src/bookmarks/commands.rs` の実装変更
- `src/media/commands.rs` の実装変更
- テストは `mockito` を使ったHTTPモック中心に移行
