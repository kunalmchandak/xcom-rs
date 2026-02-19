# doctor-command 仕様変更

## MODIFIED Requirements
### Requirement: 診断コマンドの提供
診断情報を一括取得できるコマンドを提供しなければならず、MUST である。

#### Scenario: doctorがスコープ診断と疎通結果を返す
Given 利用者が認証済みで `xcom-rs doctor --probe` を実行する
When 診断が完了する
Then 認証状態・保存先・実行モードに加えて `scopeCheck` と `apiProbe` がレスポンスに含まれる
And `apiProbe.status` と `apiProbe.durationMs` が含まれる

#### Scenario: 疎通を実行しない場合はskippedを返す
Given 利用者が `xcom-rs doctor` を `--probe` なしで実行する
When 診断が完了する
Then `apiProbe.status` は `skipped` となり、必要な次の手順が `warnings` または `authStatus.nextSteps` に含まれる
