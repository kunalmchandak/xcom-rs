## 1. CLI 追加とルーティング

- [x] 1.1 `media upload` のサブコマンドを定義する（確認: `src/cli.rs` に引数定義が追加されている）
- [x] 1.2 追加コマンドをハンドラに配線する（確認: `src/handlers/media.rs` または対応ハンドラで分岐が追加されている）

## 2. メディア処理ロジック

- [x] 2.1 ファイル存在チェックと読み取りを実装する（確認: `src/media/commands.rs` に対応関数が存在する）
- [x] 2.2 アップロード処理を実装する（確認: クライアント呼び出しが追加されている）

## 3. API クライアントとモック

- [x] 3.1 X API 呼び出しのためのクライアントIFとモック実装を追加する（確認: `src/` 配下に trait と mock が存在する）
- [x] 3.2 返却 `media_id` のフィクスチャを追加する（確認: テスト用データが `src/test_utils.rs` または専用モジュールに追加されている）

## 4. イントロスペクションとコスト

- [x] 4.1 `commands/schema/help` に新コマンドを追加する（確認: `src/introspection.rs` の一覧とSchemaが更新されている）
- [x] 4.2 コスト見積の operation key を追加する（確認: `src/billing/storage.rs` に `media.upload` が追加されている）

## 5. テスト

- [x] 5.1 アップロード成功/失敗のユニットテストを追加する（確認: `src/media/commands.rs` の `#[cfg(test)]` にテストがある）
- [x] 5.2 CLI パーステストを追加する（確認: `src/cli.rs` のテストに新サブコマンドが含まれる）
