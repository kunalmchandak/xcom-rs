# refactor-tweets-commands タスク

- [ ] 1. `tweets` コマンドの主要フロー（create/list/engagement/thread）のキャラクタライゼーションテストを追加する。
    検証: `cargo test tweets::commands::tests::` が成功する。
- [ ] 2. 引数定義・実装・テストを機能単位のモジュールへ分割する。
    検証: `cargo test tweets::commands::tests::` が成功する。
- [ ] 3. 公開 API と出力の互換性を確認する。
    検証: `cargo test tweets_integration_test` が成功する。
- [ ] 4. 全体の品質ゲートを通す。
    検証: `make check` が成功する。
