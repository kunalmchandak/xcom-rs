# refactor-introspection-registry タスク

- [ ] 1. 既存 `commands` / `schema` / `help` の挙動を固定化するキャラクタライゼーションテストを追加する。
    検証: `cargo test introspection::tests::` が成功する。
- [ ] 2. コマンドメタデータの登録元を単一化し、一覧/スキーマ/ヘルプが同一定義を参照するように分割する。
    検証: `cargo test introspection::tests::` が成功し、出力差分がない。
- [ ] 3. 既存の公開 API と出力の互換性を確認する。
    検証: 主要コマンド（`commands`, `schema`, `help`）の回帰テストが通る。
- [ ] 4. 全体の品質ゲートを通す。
    検証: `make check` が成功する。
