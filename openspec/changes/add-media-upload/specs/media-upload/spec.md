# media-upload Specification

## Purpose
メディアアップロードを追加する。

## ADDED Requirements
### Requirement: メディアアップロードコマンドの提供
`xcom-rs` は `media upload <path>` を提供し、メディアIDを取得しなければならない（MUST）。

#### Scenario: アップロード成功
- **Given** 利用者が `media upload ./image.png --output json` を実行したとき
- **When** CLIがアップロードを完了するとき
- **Then** `type="media.upload"` で `data.media_id` が返る

### Requirement: 不正パスの検出
`xcom-rs` は存在しないパスの指定を検出し、失敗を返さなければならない（MUST）。

#### Scenario: ファイルが見つからない
- **Given** 利用者が `media upload ./missing.png` を実行したとき
- **When** CLIがパスを検証するとき
- **Then** `InvalidInput` が返る
