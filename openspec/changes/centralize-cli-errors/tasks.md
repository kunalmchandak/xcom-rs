# centralize-cli-errors Tasks

- [x] 1. エラーレスポンス生成の共通ヘルパーを追加する（`ErrorDetails` と `Envelope` を組み合わせる関数/型）。
   - 検証: `src/` 内に新しいヘルパーが追加され、`cargo check` が通る。
- [x] 2. `src/main.rs` での重複エラー生成箇所をヘルパー呼び出しに置き換える。
   - 検証: `main.rs` 内の `ErrorDetails` 構築が共通ヘルパー経由になっている。
- [x] 3. 既存の失敗系テストを更新・追加し、エラー出力が変わらないことを確認する。
   - 検証: `cargo test --verbose` が成功する。
