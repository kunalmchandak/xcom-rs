## 背景
`xcom-rs` は現在、最小のRust CLI雛形のみを持ち、X API向けの機械可読プロトコル、構造化エラー、自己記述コマンドが未整備です。
AIエージェントから安全・確実・反復可能に実行できるCLIにするため、まず土台となるCLIプロトコル層を先行実装する提案です。

## 目的
- Agentic CLI Designの7原則のうち、基盤となる「機械可読」「非対話デフォルト」「観測性」「自己記述」を最初に満たす。
- 後続提案（認証/課金、投稿冪等）が依存できる共通I/O契約を確立する。

## 変更範囲
- 追加: 共通レスポンスEnvelope（`ok/type/schemaVersion/data/error/meta`）
- 追加: 構造化エラー語彙と終了コード方針（0/2/3/4）
- 追加: `commands` / `schema` / `help --output json` の自己記述機能
- 追加: `--output json|yaml|text`, `--non-interactive`, `--trace-id`, `--log-format json`

## 非対象
- 実際のX API呼び出し（tweets/users/auth実処理）
- 従量課金の実計測ロジック
- 永続ストレージ（idempotencyや予算管理）

## 成果物
- OpenSpec要件（CLI基盤能力）
- 実装タスク分解（モジュール分割と検証コマンド付き）
- 設計ドキュメント（拡張方針と後方互換ポリシー）
