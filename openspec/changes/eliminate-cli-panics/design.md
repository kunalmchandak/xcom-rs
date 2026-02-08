# eliminate-cli-panics Design

## 設計方針
- 本番コードから `unwrap` / `expect` を排除する。
- エラーは既存の `Envelope` 形式で返し、適切な終了コードを設定する。

## 構成案
- `main()` の戻り値を `anyhow::Result<()>` にするか、エラー時に明示的に `Envelope` を出力して終了する。
- 初期化失敗は `ErrorCode::InternalError` で返す。

## 代替案
- パニックを `catch_unwind` で捕捉する案
  - 根本的な解決にならず、エラーメッセージが不明瞭になる。
