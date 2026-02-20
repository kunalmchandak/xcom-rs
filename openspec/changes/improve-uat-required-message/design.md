# 設計: UAT未設定時のエラーメッセージ改善

## 方針
- **最小変更**で既存のエラーフローに乗せる。
- `403` のみで判定せず、レスポンス body のヒューリスティクスで「アプリ専用トークン」を検出する。
- `auth_required` を返し、`nextSteps` で具体的な復帰手順を提示する。

## 判定ロジック
対象: ユーザーコンテキスト必須の API で `403` が返るケース。

### 判定トリガー
- 失敗したリクエストが `GET /2/users/me` のとき
- HTTP status が `403`
- レスポンス body に以下のいずれかが含まれる（大小文字を区別しない）
  - `application-only`
  - `app-only`
  - `OAuth 2.0 Application-Only`
  - `user context`
  - `user authentication required`

### 返すエラー
- `error.code = auth_required`
- `error.message` は「UAT が必要」であることを明示する。
- `error.details.nextSteps` に以下を含める:
  - `xcom-rs auth login` の実行
  - `XCOM_RS_BEARER_TOKEN` を UAT に差し替える案内
  - `xcom-rs auth status --output json` での確認

## 実装ポイント
- `x_api` のエラー分類で body を読む仕組みを追加し、UAT 不足を識別できる構造にする。
- `/2/users/me` 失敗時に `auth_required` を返せるよう、
  `tweets` / `bookmarks` / `timeline` のエラー変換を調整する。
- 既存の `ErrorDetails::auth_required` を利用し、`nextSteps` を統一する。

## テスト方針
- `mockito` で `GET /2/users/me` に `403` + `application-only` を返す。
- 代表コマンド（例: `tweets list`）を `--output json` で実行し、
  `error.code=auth_required` と `nextSteps` を検証する。
