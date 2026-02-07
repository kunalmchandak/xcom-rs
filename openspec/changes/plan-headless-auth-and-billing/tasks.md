- [x] `auth status --output json` のレスポンス型を実装し、`authenticated` `authMode` `scopes` `nextSteps` を返す（確認: 未認証fixtureで `authenticated=false` と `nextSteps` が出る）。
- [x] `auth export` / `auth import` の入出力仕様を実装し、非対話モードで往復可能にする（確認: export結果をimportして `auth status` が復元状態を返す）。
- [x] `--non-interactive` 時の認証未完了エラーを構造化し、ブラウザ誘導の代わりに手順を返す（確認: `error.code=auth_required` と `nextSteps` が含まれる）。
- [x] `billing estimate` を実装し、操作別に `cost.credits` と `cost.usdEstimated` を返す（確認: `billing estimate tweets.create --text "hello" --output json` で両フィールドが存在）。
- [x] `--max-cost-credits` ガードを実装し、見積超過時は実行前に失敗させる（確認: 上限1で見積2以上の操作を実行した場合に `error.code=cost_limit_exceeded` を返す）。
- [x] `--budget-daily-credits` のローカル日次集計を実装する（確認: fixtureで当日累積超過時にブロックされる）。
- [x] `--dry-run` を実装し、課金ゼロで見積のみ返す（確認: `meta.cost.credits=0` と `meta.dryRun=true` が返る）。
- [x] 外部依存を排除するため、認証・課金のstub/fixtureテストを追加する（確認: ネットワーク遮断状態でもテストが成功する）。

## Acceptance #1 Failure Follow-up

- [x] `--non-interactive` の認証不足時に `error.code=AUTH_REQUIRED` と `nextSteps` を返し、終了コードを `3` に統一する（現状は `src/main.rs` の `Commands::DemoInteractive` で `INTERACTION_REQUIRED` / exit 4、`AuthCommands::Export` で `nextSteps` なし）。
- [x] `auth import` 後の状態を次回起動でも `auth status` から参照できるよう、認証ストアをプロセス間で永続化する（現状は `src/main.rs` で毎回 `AuthStore::new()` しており復元状態が保持されない）。
- [x] `--budget-daily-credits` を日次累積で判定できるように実行間で使用量を保持し、予算超過時は `DAILY_BUDGET_EXCEEDED` を返す（現状は `src/main.rs` で都度 `BudgetTracker::new()`、`src/context.rs` で `COST_LIMIT_EXCEEDED` を返している）。
- [x] `--dry-run` の応答を仕様どおり `meta.cost.credits=0` と `meta.dryRun=true` にする（現状は `src/main.rs` で `data.cost.credits=0` のみ、`meta.cost` なし）。
- [x] `tests/auth_billing_test.rs` の未実装プレースホルダ（`test_auth_export_import_roundtrip` / `test_budget_daily_credits_tracking`）を実検証テストに置き換え、仕様シナリオをCLIフローで検証する。
