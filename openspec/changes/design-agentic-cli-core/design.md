## 概要
この提案は、`xcom-rs` を「人間向けUI」ではなく「エージェント向け実行プロトコル」として成立させるための基盤設計を定義します。
後続のAPI実装より先に、I/O契約を固定し、コマンドの自己記述を可能にします。

## 設計方針
- 出力は `stdout` に結果のみ、`stderr` にログのみを出す。
- すべてのコマンドは共通Envelopeで返す。
- エラーも必ず構造化して返し、機械的なリトライ判断を可能にする。
- `schemaVersion` をトップレベル固定し、将来の互換性管理を行う。

## 想定モジュール
- `cli`: `clap` によるサブコマンド定義
- `protocol`: Envelope型、Error型、ExitCode変換
- `introspection`: `commands/schema/help` のJSON生成
- `logging`: `tracing` のjson/text切替（stderr出力）

## 互換性ポリシー
- `schemaVersion` は初期値 `1`。
- 破壊的変更は `schemaVersion` を増加し、旧形式は明示的にサポート期限を示す。
- `error.code` は列挙語彙として管理し、意味変更を避ける。

## トレードオフ
- 初期段階で自己記述を入れるため、実装コストは増える。
- 一方で後続機能追加時の再設計コストを大幅に抑制でき、LLM運用時の失敗率も下げられる。

## 外部依存への対応（mock-first）
- この提案範囲では外部API接続を持たない。
- `commands/schema/help` のテストはローカル実行のみで完結させる。
