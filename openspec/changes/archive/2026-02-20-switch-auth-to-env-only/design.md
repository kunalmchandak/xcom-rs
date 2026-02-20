## 概要
認証トークンの投入経路を環境変数に限定し、CLI の引数や import/export による持ち込みを廃止する。履歴への露出や取り扱いミスを避けるため、トークンは常に環境変数から読み出す。

## 認証入力の設計
- 必須: `XCOM_RS_BEARER_TOKEN`
  - 値は生トークン、または `Bearer <token>` 形式を許容する。
  - 認証状態の判定はこの変数の有無で行う。
- 任意: `XCOM_RS_SCOPES`
  - スコープ診断用。空白区切りを基本とし、カンマ区切りも許容する。
  - 未設定の場合、スコープ診断は「未実施」として扱う。
- 任意: `XCOM_RS_EXPIRES_AT`
  - UNIX epoch 秒。現在時刻が `>=` の場合は未認証扱いとする。
  - 不正な値は未認証扱いにし、`nextSteps` で設定ミスを案内する。

## セキュリティ方針
- トークン文字列をログ、`auth status`、`doctor` の出力に含めない。
- トークン長や先頭数文字の出力も禁止する。

## コマンド設計
- `auth status` のみ提供する。
- `auth import/export` は廃止。
- `AuthStore` は永続化を行わず、環境変数を解決する薄い resolver とする。

## エラー・nextSteps の統一
認証不足時の nextSteps は以下を標準文面とし、全コマンドで統一する。
- `Set XCOM_RS_BEARER_TOKEN and re-run the command`
- （必要に応じて）`Set XCOM_RS_SCOPES to enable scope diagnostics`
- （期限切れ時）`Verify XCOM_RS_EXPIRES_AT is not in the past`

## doctor の診断出力
- `authStoragePath` は env-only 化により不要となるため、出力しない。
- `scopeCheck` は `XCOM_RS_SCOPES` が未設定のときはスキップし、警告に反映する。

## テスト方針
- env-only 認証を前提にしたフィクスチャテストを追加する。
- `auth import/export` に依存する既存テストは削除・置換する。
