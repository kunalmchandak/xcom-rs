# storage-path-resolution

## MODIFIED Requirements

### Requirement: 保存先パスの解決規則

認証情報と予算トラッカーの保存先はユーザーデータ領域に解決されなければならない (MUST)。

#### Scenario: XDG_DATA_HOMEが設定されている場合の解決
- Given `XDG_DATA_HOME`が設定されている
- When `auth.json`または`budget.json`の保存先を解決する
- Then `XDG_DATA_HOME/xcom-rs/<file>.json`が選択される

#### Scenario: XDG環境変数が未設定の場合のフォールバック
- Given `XDG_DATA_HOME`が設定されていない
- When `auth.json`の保存先を解決する
- Then `~/.local/share/xcom-rs/auth.json`が選択される
- And `budget.json`の保存先を解決する
- Then `~/.local/share/xcom-rs/budget.json`が選択される
