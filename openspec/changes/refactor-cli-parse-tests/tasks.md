# refactor-cli-parse-tests タスク

- [ ] 1. 現行 CLI のパース結果を固定化するキャラクタライゼーションテストを追加する。
    検証: `cargo test cli::tests::` が成功する。
- [ ] 2. テーブル駆動テストに置き換え、重複を削減する。
    検証: `cargo test cli::tests::` が成功する。
- [ ] 3. テスト配置の整理（専用モジュール化）を行う。
    検証: `cargo test cli::tests::` が成功する。
- [ ] 4. 全体の品質ゲートを通す。
    検証: `make check` が成功する。
