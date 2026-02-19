# 設計: add-media-upload

## 方針
- 単一ファイルのワンショットアップロードのみを対象とする。
- ファイルの存在と読み取り可能性を事前に検証する。

## API マッピング
- upload: `POST /2/media/upload`

## 入出力設計
- 入力は `media upload <path>` のみ。
- `type` は `media.upload` を使用する。
- 出力は `data.media_id` を返す。

## エラー設計
- パスが存在しない場合は `InvalidInput` を返す。
- 認証不足は `AuthRequired` を返す。

## イントロスペクションとコスト
- `commands/schema/help` に新コマンドを追加する。
- コスト見積の operation key に `media.upload` を追加する。

## テスト・モック
- X API クライアントのモックで `media_id` を固定値で返す。
