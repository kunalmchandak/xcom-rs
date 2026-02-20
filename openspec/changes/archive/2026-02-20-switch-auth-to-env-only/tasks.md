## 1. CLI 仕様の整理

- [x] 1.1 `auth import/export` コマンド定義とハンドラを削除する（確認: `src/cli.rs` と `src/handlers/auth.rs` から `Import/Export` が消える）
- [x] 1.2 `auth status` のレスポンス生成を env-only に切り替える（確認: `src/auth/storage.rs` が環境変数から判定する実装になる）

## 2. 認証判定・診断の整合

- [x] 2.1 認証不足時の nextSteps を環境変数案内に統一する（確認: `src/auth/storage.rs`/`src/handlers/timeline.rs`/`src/timeline/commands.rs` に `XCOM_RS_BEARER_TOKEN` 案内が入る）
- [x] 2.2 `doctor` の auth 保存先診断を廃止し、env-only に合わせて出力を調整する（確認: `src/doctor.rs` で `authStoragePath` が出力されない）
- [x] 2.3 任意のスコープ診断用 env (`XCOM_RS_SCOPES`) と期限 (`XCOM_RS_EXPIRES_AT`) の取り扱いを追加する（確認: `auth status` と `doctor` が未設定時にスキップ/警告を返す）

## 3. テスト更新

- [x] 3.1 `auth import/export` 依存の統合テストを削除し、env-only の認証テストに置換する（確認: `tests/auth_billing_test.rs` が `XCOM_RS_BEARER_TOKEN` を用いた検証に更新される）
- [x] 3.2 auth の XDG パス検証を削除し、予算トラッカーのみを残す（確認: `tests/xdg_paths_test.rs` の auth 保存先テストが削除される）

## 4. ドキュメント整合

- [x] 4.1 README の Quick Start と Authentication 節を env-only に更新する（確認: `README.md` に `auth import/export` の記述がない）
- [x] 4.2 OpenSpec の参照ドキュメントを更新する（確認: `openspec/specs/docs-quickstart/spec.md` のシナリオが env-only になる）

## 5. 回帰確認

- [x] 5.1 主要テストが通ることを確認する（確認: `cargo test --verbose`）

## Acceptance #1 Failure Follow-up

- [x] `timeline home`/`timeline mentions` が env-only 認証に未対応。`src/timeline/commands.rs` の `resolve_me` が `XCOM_AUTHENTICATED` を参照しており、`XCOM_RS_BEARER_TOKEN` を設定しても `auth_required` になるため、`AuthStore` または `XCOM_RS_BEARER_TOKEN` 参照へ切り替える。
- [x] `cargo test --verbose` が Doc-tests で失敗しているため修正する。現状 `src/logging.rs:2` で `tracing_subscriber`、`src/tweets/ledger.rs:2` で `rusqlite` が `can't find crate` となりテストゲートを通過できない（`tool-output/tool_c7947b816001Yh59I1fOSSEh5O:440-455`）。
- [x] auth import/export 廃止後も import 計画用の未使用型が残っているため削除する。`src/auth/models.rs` の `ImportAction`/`ImportPlan`/`AuthToken` と `src/auth/mod.rs` の再エクスポートを見直し、env-only フローで使われない死蔵コードを解消する。
