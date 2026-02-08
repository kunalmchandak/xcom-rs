# 設計: 認証・予算の保存先をデータ領域へ統一

## 方針
- 認証情報と予算トラッカーはいずれもユーザーデータとして扱う
- 保存先は`XDG_DATA_HOME`配下に統一し、未設定時は`~/.local/share`へフォールバックする

## パス解決規則
- 認証(`auth.json`):
  - `XDG_DATA_HOME/xcom-rs/auth.json`
  - フォールバック: `~/.local/share/xcom-rs/auth.json`
- 予算(`budget.json`):
  - `XDG_DATA_HOME/xcom-rs/budget.json`
  - フォールバック: `~/.local/share/xcom-rs/budget.json`

## 互換性
- 既存の`~/.config/xcom-rs/*.json`は自動移行しない
- 移行が必要な場合は別変更で明示的な移行コマンドを検討する
