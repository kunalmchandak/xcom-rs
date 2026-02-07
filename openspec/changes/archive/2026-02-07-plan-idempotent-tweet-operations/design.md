## 概要
この提案は `tweets` 操作に対して、再実行可能性とコンテキスト効率を同時に満たすための設計です。
失敗時の再実行を安全に行うため、ローカル台帳を導入します。

## 冪等性設計
- `tweets create` は `--client-request-id` を受け付ける。
- 未指定時はCLIがUUIDを生成して `meta.clientRequestId` に返す。
- ローカル台帳（sqlite推奨）に `client_request_id -> request_hash -> tweet_id -> status` を保存する。
- タイムアウト後の再実行時は台帳を参照し、既存成功結果を返す。
- 競合時は `--if-exists return|error` を選択可能にする。

## 取得系最適化
- `tweets list` は `--fields` で投影し、既定では要約項目のみ返す。
- `--limit` `--cursor` で明示ページングを行う。
- `--output ndjson` で逐次処理可能にする。

## モック戦略
- 投稿成功/タイムアウト/429を再現するHTTPモックを用意する。
- 台帳はテスト専用DBで検証し、外部X APIキーなしで再実行フローを試験する。
- リトライ可能エラー（429/5xx）をfixture化する。

## 運用上の注意
- 台帳サイズはTTLでガベージコレクションする。
- request_hash不一致時は再利用せず、誤関連付けを防ぐ。
