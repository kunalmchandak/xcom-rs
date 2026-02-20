# doctor-command Specification

## Purpose
TBD - created by archiving change add-doctor-diagnostics. Update Purpose after archive.
## Requirements
### Requirement: 診断コマンドの提供
診断情報を一括取得できるコマンドを提供しなければならず、MUST である。

#### Scenario: doctorコマンドが診断情報を返す
Given `xcom-rs doctor` が実行される
When 実行時設定の取得に成功する
Then 認証状態・実行モードが1つのレスポンスに含まれる
And 予算トラッカーの保存先が含まれる
And 認証の保存先は出力に含まれない

