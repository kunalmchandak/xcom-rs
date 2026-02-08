# centralize-cli-errors Design

## 設計方針
- エラーレスポンスの生成を単一箇所へ集約する。
- `traceId` などのメタ情報付与はヘルパー側で一貫して処理する。
- 既存のエラーコードと終了コードの挙動は維持する。

## 構成案
- `response` もしくは `errors` モジュールに `ErrorResponder` を追加。
- `ErrorResponder::emit(error_details, output_format, meta)` のような API を提供する。

## 代替案
- `main.rs` 内で関数に切り出す案
  - 局所改善に留まり、将来的な利用箇所拡張が難しい。
