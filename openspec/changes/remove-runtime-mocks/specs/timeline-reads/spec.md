## MODIFIED Requirements

### Requirement: ホームタイムラインの取得
`xcom-rs` は `timeline home` の結果をX APIの応答に基づいて返し、擬似エラー注入やモック実装による結果改変を行ってはならない（MUST NOT）。

#### Scenario: 擬似エラー注入の無効化
- **Given** `XCOM_SIMULATE_ERROR` が設定されている
- **When** 利用者が `timeline home --output json` を実行する
- **Then** 返却内容はAPIレスポンスに基づく
