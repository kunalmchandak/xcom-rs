# docs-quickstart 仕様変更

## MODIFIED Requirements
### Requirement: Quick Startはdoctor起点の最短導線を示す
READMEのQuick Startは、インストール・認証・doctor・代表操作の順で実行例を示さなければならない（MUST）。

#### Scenario: Quick Startに必須ステップが記載される
Given 利用者がREADMEのQuick Startを参照する
When Quick Startの手順を確認する
Then `XCOM_RS_BEARER_TOKEN` の設定と `xcom-rs doctor` と代表操作（例: `xcom-rs tweets create`）が順に記載される
