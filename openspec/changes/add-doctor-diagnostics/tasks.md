1. [x] CLIに `doctor` サブコマンドを追加する
   - 変更箇所: `src/cli.rs`
   - 検証: `xcom-rs doctor --output json` がコマンドとして認識される
2. [x] 診断データ構造と収集ロジックを実装する
   - 変更箇所: 新規モジュールまたは `src/auth.rs`, `src/billing.rs`, `src/context.rs`
   - 検証: `doctor` のJSON出力に認証状態/保存先/実行モードが含まれる
3. [x] 出力Envelopeに統合する
   - 変更箇所: `src/main.rs`
   - 検証: `ok=true` かつ `type=doctor` のレスポンスを返す
4. [x] 失敗時のnextStepsを整備する
   - 変更箇所: `src/protocol.rs` または `doctor` 実装
   - 検証: 取得不能時に `error.details.nextSteps` が含まれる
5. [x] 単体テストを追加する
   - 変更箇所: `src/` の該当モジュール、`tests/` など
   - 検証: `cargo test doctor` がパスする
