## 1. CLI 追加とルーティング

- [x] 1.1 `timeline home/mentions/user` のサブコマンドを定義する（確認: `src/cli.rs` に `TimelineCommands` enum が追加されている）
- [x] 1.2 `timeline` ハンドラを追加してルーティングする（確認: `src/handlers/timeline.rs` と `src/handlers/mod.rs`、`src/main.rs` が更新されている）

## 2. タイムライン取得ロジック

- [x] 2.1 認証ユーザーID取得の処理を追加する（確認: `src/timeline/commands.rs` の `resolve_me()` 関数が `XCOM_TEST_USER_ID` / `XCOM_AUTHENTICATED` 環境変数でモック化されている）
- [x] 2.2 home/mentions/user の取得処理を実装する（確認: `src/timeline/commands.rs` の `get_home()`, `get_mentions()`, `get_user_tweets()` が存在する）

## 3. API クライアントとモック

- [x] 3.1 タイムライン用APIクライアントのモックを追加する（確認: `XCOM_SIMULATE_ERROR` 環境変数によるモックが `src/timeline/commands.rs` に追加されている）
- [x] 3.2 ページングトークンのフィクスチャを追加する（確認: テストで `next_token` が `test_timeline_pagination_next_token` と `test_timeline_pagination_with_previous_token` で検証されている）

## 4. イントロスペクションとコスト

- [x] 4.1 `commands/schema/help` に timeline コマンドを追加する（確認: `src/introspection.rs` に `timeline.home`, `timeline.mentions`, `timeline.user` が追加されている）
- [x] 4.2 コスト見積の operation key を追加する（確認: `src/billing/storage.rs` に `timeline.home`, `timeline.mentions`, `timeline.user` が追加されている）

## 5. テスト

- [x] 5.1 タイムライン取得のユニットテストを追加する（確認: `src/timeline/commands.rs` の `#[cfg(test)]` に9件のテストが追加）
- [x] 5.2 CLI パーステストを追加する（確認: `src/cli.rs` のテストに `test_timeline_home_command`, `test_timeline_mentions_command`, `test_timeline_user_command` 等が含まれる）
