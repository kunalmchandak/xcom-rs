# refactor-introspection-registry 設計

## 設計方針
- コマンド定義を「登録表」として集約し、一覧/スキーマ/ヘルプ生成が同一データを参照する。
- 出力の公開仕様は維持し、内部の構成のみを変更する。

## 構成案
- `introspection/registry.rs`: コマンドメタデータの定義
- `introspection/commands.rs`: `commands` 出力生成
- `introspection/schema.rs`: `schema` 生成
- `introspection/help.rs`: `help` 生成
- `introspection/mod.rs`: 既存 API を維持して再公開

## 代替案
- 現状の単一ファイルのまま拡張する
  - 変更箇所が増え続け、レビューと保守のコストが高い
