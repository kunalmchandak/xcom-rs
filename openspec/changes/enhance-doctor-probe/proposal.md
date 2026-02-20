## Why

`doctor --probe` はTCP疎通のみでHTTPステータスを実際に確認しておらず、APIの健全性が判定できません。実APIへの軽量HTTPプローブを追加します。

## What Changes

- `doctor --probe` がHTTPSリクエストを送信し、HTTPステータスを返す
- 認証情報がない場合はスキップ理由を明示する
- モック可能なプローブ実装にしてテスト可能にする

## Capabilities

### New Capabilities
- なし

### Modified Capabilities
- `doctor-command`: 実HTTPプローブの追加と結果の詳細化

## Impact

- `src/handlers/doctor.rs` のプローブ実装変更
- `src/doctor.rs` の診断出力の詳細化
