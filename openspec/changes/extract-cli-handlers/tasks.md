# extract-cli-handlers Tasks

1. `src/handlers/mod.rs` と各ドメイン用ハンドラーの雛形を追加する（例: `handlers::auth`, `handlers::billing`, `handlers::tweets`, `handlers::introspection`, `handlers::doctor`）。
   - 検証: `src/handlers/` 配下にモジュールが存在し、`cargo check` が通る。
2. `src/main.rs` の `match cli.command` から各ドメイン処理を移動し、ハンドラー呼び出しのみ残す。
   - 検証: `src/main.rs` にコマンド固有ロジックが残らず、各ハンドラーへの委譲になっている。
3. ハンドラー側で出力フォーマットと `Envelope` 生成が現行と一致するように移植する。
   - 検証: 既存の統合テストが同じ出力を確認できる（`cargo test --verbose`）。
4. 変更に合わせてテストを更新する（必要な場合のみ）。
   - 検証: 影響するテストがすべて成功する（`cargo test --verbose`）。
