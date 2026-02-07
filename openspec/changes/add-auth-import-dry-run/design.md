# 設計メモ

## 変更計画モデル
- `plannedActions`: create/update/skip/fail のいずれか
- 失敗時は `reason` を必須で保持

## dry-runの挙動
- 解析と検証のみを実施し、保存ファイルには書き込まない
- 実行時の出力は通常実行と同等のサマリを返す

## 出力形式
- `--output json` の場合は計画をJSONで返す
- `text` の場合は簡易サマリ + 詳細一覧
