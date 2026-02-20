## Why

検索/タイムライン/取得系コマンドがスタブのため、実際のX APIデータを返せません。読み取り系の実API呼び出しへ移行し、ページングやフィールド指定を正しく反映します。

## What Changes

- `search recent/users` を実API呼び出しへ置き換える
- `timeline home/mentions/user` を実API呼び出しへ置き換える
- `tweets list/show/conversation` を実API呼び出しへ置き換える
- APIレスポンスのページング/フィールド投影をCLI出力へ反映する

## Capabilities

### New Capabilities
- なし

### Modified Capabilities
- `search-commands`: 実API検索の取得に変更
- `timeline-reads`: 実APIのタイムライン取得に変更
- `tweets-query-efficiency`: フィールド投影/ページングをAPI由来で反映
- `tweets-replies`: `tweets show/conversation` を実API化

## Impact

- `src/search/commands.rs` / `src/timeline/commands.rs` / `src/tweets/commands/list.rs` / `src/tweets/commands/show.rs` の実装変更
- モックサーバベースのテスト追加
