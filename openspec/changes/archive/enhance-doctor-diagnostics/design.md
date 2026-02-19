# enhance-doctor-diagnostics 設計メモ

## 追加する診断項目
- scopeCheck: 操作別に必要スコープの不足を列挙
- apiProbe: `status`（`ok` | `failed` | `skipped`）、`durationMs`、`error`（任意）

## 実行条件
- `--probe` 指定時のみネットワーク疎通を試行する
- 認証未設定時は `apiProbe.status=skipped` とし、次の手順を返す

## 実装方針
- プローブ用のインターフェースを用意し、実装とモックを差し替え可能にする
- 既存のdoctor収集フローにオプションで組み込む

## テスト方針
- モックプローブで `ok`/`failed`/`skipped` を網羅する
- 外部APIキーは不要とする
