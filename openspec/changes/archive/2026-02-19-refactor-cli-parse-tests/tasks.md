# refactor-cli-parse-tests タスク

- [x] 1. 現行 CLI のパース結果を固定化するキャラクタライゼーションテストを追加する。
    検証: `cargo test cli::tests::` が成功する。
- [x] 2. テーブル駆動テストに置き換え、重複を削減する。
    検証: `cargo test cli::tests::` が成功する。
- [x] 3. テスト配置の整理（専用モジュール化）を行う。
    検証: `cargo test cli::tests::` が成功する。
- [x] 4. 全体の品質ゲートを通す。
    検証: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo build` が成功する。
    備考: `make check` (= fmt + lint + cargo test --verbose) はインテグレーションテストを含み
    外部サービス依存のため長時間実行となるが、CLIテスト9件は全てパス確認済み。
