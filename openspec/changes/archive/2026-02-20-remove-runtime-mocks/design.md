## Context

`tweets list/show/conversation` にモック実装が残っており、実行時にダミー生成が返るリスクがある。また `XCOM_SIMULATE_ERROR` による擬似エラー注入が本番経路に残っているため、実APIの挙動と乖離する可能性がある。テストは外部APIに依存しないよう `mockito` によるHTTPスタブで検証可能な構成を維持する必要がある。

## Goals / Non-Goals

**Goals:**

- 実行時にモック実装・ダミー生成・擬似エラー注入を一切使用しない
- `tweets`/`timeline` の実行結果はX API応答にのみ基づく
- テストは `mockito` によるHTTPモックで完結し、外部API不要のまま維持する
- テスト用ユーティリティを本番ビルドから分離する

**Non-Goals:**

- `mockito` などテスト用HTTPモック基盤の廃止
- 実APIを使ったE2Eテストの追加
- 既存のレスポンス形式やCLIインターフェースの変更

## Decisions

- **モッククライアントの削除**: `MockTweetApiClient` を削除し、`TweetCommand` は実APIクライアントの注入を必須とする。ダミー生成経路を根絶し、実運用の誤動作を防ぐ。
- **擬似エラー注入の廃止**: `XCOM_SIMULATE_ERROR` / `XCOM_RETRY_AFTER_MS` を削除し、エラーはHTTPレスポンス分類のみで返す。テストは `mockito` でHTTPレスポンスを再現する。
- **テスト分離**: `src/test_utils.rs` のフィクスチャ類は `tests/` 配下へ移動し、本番ビルドへの露出をなくす。テスト利用性は維持する。
- **タイムラインのHTTPモック対応**: タイムライン取得のテストが `mockito` を利用できるよう、HTTPアクセス先の差し替え手段を設ける（例: 環境変数のベースURL）。擬似エラー注入の削除後もテストの検証性を確保する。

## Risks / Trade-offs

- **[Risk] テストの再構築コスト増** → **Mitigation**: `mockito` を利用したHTTPモックテストに段階的に置換し、既存テストの意図を維持する。
- **[Risk] タイムラインのベースURL導入が変更範囲を広げる** → **Mitigation**: 変更はテスト用途に限定し、デフォルトは現行API URLを維持する。
- **[Risk] モック削除で認証未設定時の挙動が厳格化** → **Mitigation**: `auth_required` と `nextSteps` の既存設計に準拠したエラーを返す。
