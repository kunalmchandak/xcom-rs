# refactor-introspection-registry タスク

- [x] 1. 既存 `commands` / `schema` / `help` の挙動を固定化するキャラクタライゼーションテストを追加する。
    検証: `cargo test introspection::tests::` が成功する。
- [x] 2. コマンドメタデータの登録元を単一化し、一覧/スキーマ/ヘルプが同一定義を参照するように分割する。
    検証: `cargo test introspection::tests::` が成功し、出力差分がない。
- [x] 3. 既存の公開 API と出力の互換性を確認する。
    検証: 主要コマンド（`commands`, `schema`, `help`）の回帰テストが通る。
- [x] 4. 全体の品質ゲートを通す。
    検証: `make check` が成功する。

## Acceptance Notes (2026-02-19)

- 実測: `make check` が成功（fmt/clippy/lib 182件/doc 1件）。
- doctest: `cargo test --doc --verbose` は `src/context.rs` の 1 件のみ実行される（`src/logging.rs` / `src/tweets/ledger.rs` はドキュメント内に Rust のコードブロックがないため対象外）。
- 依存解決: `rustdoc` 実行に `--extern tracing_subscriber` / `--extern rusqlite` が渡っているため、`E0463: can't find crate` は再現しない。
- 以前報告されていた `E0463`（`src/logging.rs:2`, `src/tweets/ledger.rs:2`）は本ワークツリーでは非再現。別ブランチ/古いログ/別ディレクトリ由来の可能性が高い。
- Makefile: `make test` は lib/doc に分離済みで、integration tests は `make test-integration` に分離済み（長時間実行対策）。

## Acceptance #1 Failure Follow-up

- [ ] Git working tree が dirty です。`git status --porcelain` が空になるように、`openspec/changes/refactor-introspection-registry/tasks.md` の変更をコミットするか、意図しない変更なら差し戻してください。
