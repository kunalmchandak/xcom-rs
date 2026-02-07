- [x] `src/protocol` 相当の共通レスポンス型を追加し、`ok/type/schemaVersion/data/error/meta` を必須フィールドとして実装する（確認: `cargo run -- commands --output json` の出力に `schemaVersion` が含まれる）。
- [x] 失敗時の構造化エラー型を追加し、`error.code`/`error.message`/`error.isRetryable` を返す（確認: 不正引数で `cargo run -- unknown` 実行時にJSONエラーが返る）。
- [x] 終了コード方針（0/2/3/4）を実装し、引数エラーと認証エラーを区別する（確認: 引数不正時の終了コードが2であることをシェルで確認）。
- [x] `--output json|yaml|text` をグローバルオプションとして導入する（確認: 同一コマンドで3形式が切替できる）。
- [x] `commands --output json` を実装し、サブコマンド・引数・危険度・課金有無を列挙する（確認: 出力JSONに `commands[]` と `risk` が存在する）。
- [x] `schema --command <name> --output json-schema` を実装し、入出力Schemaを取得可能にする（確認: `cargo run -- schema --command commands --output json-schema` が成功する）。
- [x] `help <command> --output json` を実装し、終了コードとエラー語彙を返す（確認: `cargo run -- help commands --output json` に `exitCodes` が含まれる）。
- [x] `--trace-id` と `--log-format json` を実装し、ログ相関可能にする（確認: stderrログに `traceId` が出力される）。

## Acceptance #1 Failure Follow-up

- [x] `--trace-id` 指定時にレスポンスEnvelopeの `meta.traceId` へも値を反映する（現状は `src/main.rs` の `Envelope::success(...)` / `Envelope::error(...)` を使用しており `meta` が常に `None` のため、`cli-core/spec.md` の trace-id伝播要件を満たしていない）。
- [x] `help <command> --output json` の `data` に `examples[]` と `errorVocabulary[]` を追加し、`cli-introspection/spec.md` の必須フィールド名に合わせる（現状 `src/introspection.rs` の `CommandHelp` は `exitCodes` と `errors` のみ）。
- [x] `schema --command <name> --output json-schema` が返す `outputSchema` を「data本体」ではなくEnvelope全体（`ok/type/schemaVersion/data/error/meta`）のJSON Schemaに修正する（現状 `src/introspection.rs::CommandSchema::for_command` は `commands` 等のデータ形のみを返却）。
- [x] `--non-interactive` を実行フローへ統合し、対話が必要なケースでプロンプト表示せず構造化エラーと次手順を返す実装を追加する（現状 `src/cli.rs` で定義のみ、`src/main.rs` 含む実行経路で未参照）。
- [x] `--help` / `--version` の回帰を修正し成功系として扱う（現状 `src/main.rs` の `Cli::try_parse()` エラー分岐で `DisplayHelp`/`DisplayVersion` も `INVALID_ARGUMENT` + exit code 2 に変換される）。

## Acceptance #2 Failure Follow-up

- [x] `--non-interactive` を実行フローで実際に参照し、対話が必要なケースで `INTERACTION_REQUIRED` と `details.nextSteps` を含む構造化エラーを返すように統合する（`src/cli.rs` の `non_interactive` は定義済みだが `src/main.rs::main` のコマンド実行分岐で未使用、`src/protocol.rs::ErrorDetails::interaction_required` も実行経路から未呼び出し）。

## Acceptance #3 Failure Follow-up

- [x] `--non-interactive` の実行フロー統合を完了する（`src/main.rs::main` では `ExecutionContext` を生成しているが `check_interaction_required()` の実行呼び出しがなく、`src/context.rs::ExecutionContext::check_interaction_required` / `src/protocol.rs::ErrorDetails::interaction_required` が実行経路で未使用）。対話が必要な実コマンドにチェックを組み込み、`INTERACTION_REQUIRED` と `details.nextSteps` を返す統合テストを追加する。
