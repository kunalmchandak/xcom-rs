# centralize-cli-errors Tasks

- [x] 1. エラーレスポンス生成の共通ヘルパーを追加する（`ErrorDetails` と `Envelope` を組み合わせる関数/型）。
   - 検証: `src/` 内に新しいヘルパーが追加され、`cargo check` が通る。
- [x] 2. `src/main.rs` での重複エラー生成箇所をヘルパー呼び出しに置き換える。
   - 検証: `main.rs` 内の `ErrorDetails` 構築が共通ヘルパー経由になっている。
- [x] 3. 既存の失敗系テストを更新・追加し、エラー出力が変わらないことを確認する。
   - 検証: `cargo test --verbose` が成功する。

## Acceptance #1 Failure Follow-up

- [x] `src/main.rs` の `main` 関数で `ErrorDetails::new` / `with_retry_after` / `with_details` / `auth_required` が直接構築されており（例: `src/main.rs:37`, `src/main.rs:49`, `src/main.rs:187`, `src/main.rs:485`）、`Envelope` と `ErrorDetails` の共通生成経路要件を満たしていないため、`ErrorResponder` 側に `ErrorDetails` 生成を含む共通APIを追加して全失敗分岐を移行する。
- [x] `src/main.rs:66`-`src/main.rs:67` の `if_exists` パース失敗分岐が `eprintln!` + `std::process::exit` のままで `ErrorResponder` を経由しておらず、失敗レスポンスの統一経路から外れているため、構造化エラー応答（既存フォーマット維持）に置き換える。
