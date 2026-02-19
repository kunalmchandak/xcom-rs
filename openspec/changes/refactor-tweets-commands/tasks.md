# refactor-tweets-commands タスク

- [x] 1. `tweets` コマンドの主要フロー（create/list/engagement/thread）のキャラクタライゼーションテストを追加する。
    検証: `cargo test tweets::commands::tests::` が成功する。
    実施内容: `commands/create.rs`, `commands/list.rs`, `commands/engagement.rs`, `commands/thread.rs`, `commands/show.rs` に各フロー（正常系・異常系・エラー分類）を網羅したキャラクタライゼーションテストを追加した。
- [x] 2. 引数定義・実装・テストを機能単位のモジュールへ分割する。
    検証: `cargo test tweets::commands::tests::` が成功する。
    実施内容: `src/tweets/commands.rs` (1300行) を以下のモジュールに分割した:
    - `commands/types.rs` — 全型定義（引数・結果・エラー型）
    - `commands/create.rs` — create実装＋テスト
    - `commands/list.rs` — list実装＋テスト
    - `commands/engagement.rs` — like/unlike/retweet/unretweet実装＋テスト
    - `commands/thread.rs` — reply/thread実装＋テスト
    - `commands/show.rs` — show/conversation実装＋テスト
    - `commands/mod.rs` — TweetCommandファサード＋全型の再エクスポート＋テスト
- [x] 3. 公開 API と出力の互換性を確認する。
    検証: `cargo test --test tweets_integration_test` が成功する。
    実施内容: 統合テスト6件（test_timeout_retry_with_ledger, test_different_content_same_client_request_id, test_if_exists_error_policy, test_ndjson_output_format, test_field_projection, test_error_classification）が全て通ることを確認した。
- [x] 4. 全体の品質ゲートを通す。
    検証: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test` が成功する。
    実施内容: フォーマット・lint・ユニットテスト198件が全て成功することを確認した。
