# docs-quickstart 仕様変更

## Requirements
### Requirement: Quick Startはdoctor起点の最短導線を示す
READMEのQuick Startは、インストール・認証・doctor・代表操作の順で実行例を示さなければならない（MUST）。

#### Scenario: Quick Startに必須ステップが記載される
Given 利用者がREADMEのQuick Startを参照する
When Quick Startの手順を確認する
Then 環境変数設定（`XCOM_RS_BEARER_TOKEN`）と `xcom-rs auth status` と `xcom-rs doctor` と代表操作（例: `xcom-rs tweets create`）が順に記載される

### Requirement: READMEのFeaturesとdocs/examplesはCLI実装に整合する
READMEのFeaturesとdocs/examplesは、現行CLIの主要コマンド群に整合していなければならない（MUST）。

#### Scenario: 主要コマンドが例示される
Given 利用者がREADMEとdocs/examplesを読む
When 機能一覧と実行例を確認する
Then `tweets thread` `tweets reply` `timeline` `search` `bookmarks` `media upload` `doctor` が含まれる
