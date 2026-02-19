# add-shell-completions 設計メモ

## 方針
- clapの補完生成を利用し、stdoutへ出力する。
- サブコマンド名は `completion` とし `--shell` を必須にする。

## 出力仕様
- 生成結果はstdoutのみ
- 失敗時は既存のエラーエンベロープに従う
