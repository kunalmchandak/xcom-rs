# 変更提案: auth/budget保存先を.local/shareへ

## 背景
認証情報(`auth.json`)と予算トラッカー(`budget.json`)はgit管理すべき情報ではなく、
ユーザーデータ領域へ保存されるべきです。現在は`authStoragePath`が
`~/.config/xcom-rs/auth.json`を指し、`budgetStoragePath`も
フォールバックが`~/.config/xcom-rs/budget.json`になっています。

## 目的
- 認証・予算の保存先をユーザーデータ領域(`.local/share`)に統一する
- `doctor`の出力(`authStoragePath`/`budgetStoragePath`)が新しい解決規則を反映する

## スコープ
- `auth.json`のデフォルト保存先の解決規則を`XDG_DATA_HOME`基準へ変更
- `budget.json`のフォールバックを`~/.local/share`へ変更
- ストレージ解決規則の仕様を更新
- 関連テストの期待値を更新

## 非スコープ
- 既存ファイルの自動移行(コピー/移動)
- 既存ユーザーデータのクリーンアップ

## 影響
- 既存の`~/.config/xcom-rs/auth.json`は自動で読み込まれない
- `doctor`の`authStoragePath`/`budgetStoragePath`表示が変更される

## リスク/対応
- 既存の認証情報が見つからない可能性
  - 対応: 明示的な再ログイン/再インポートが必要
