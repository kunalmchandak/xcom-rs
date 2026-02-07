# doctor-command

## ADDED Requirements

### Requirement: 診断コマンドの提供
診断情報を一括取得できるコマンドを提供しなければならず、MUST である。

#### Scenario: doctorコマンドが診断情報を返す
Given `xcom-rs doctor` が実行される
When 実行時設定と保存先の解決に成功する
Then 認証状態・保存先・実行モードが1つのレスポンスに含まれる

#### Scenario: 部分失敗時も診断結果を返す
Given 一部の診断情報取得に失敗する
When `xcom-rs doctor` が実行される
Then 取得できた情報は `data` に含まれ、失敗内容は明示される
