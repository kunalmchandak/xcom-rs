# install-skills Specification

## Purpose
TBD - created by archiving change add-install-skills-command. Update Purpose after archive.
## Requirements
### Requirement: install-skills サブコマンドの提供
`xcom-rs` は埋め込みスキルをインストールする `install-skills` サブコマンドを提供しなければならない（MUST）。

#### Scenario: プロジェクトスコープでのスキル導入
- **Given** 利用者が `xcom-rs install-skills --agent claude --yes` を実行したとき
- **When** CLI がインストール処理を完了するとき
- **Then** `.agents/skills/<skill-name>/SKILL.md` が生成され、`.claude/skills/<skill-name>` が作成される

### Requirement: 埋め込みスキル定義の配置
`xcom-rs` が探索対象とする埋め込みスキルは、リポジトリ内の `skills/<skill-name>/SKILL.md` として存在しなければならない（MUST）。

#### Scenario: スキル定義ファイルの存在確認
- **Given** 利用者が `xcom-rs install-skills --non-interactive --json` を実行したとき
- **When** CLI が埋め込みスキルを探索するとき
- **Then** `skills/<skill-name>/SKILL.md` が存在するスキルのみを有効な候補として扱う

### Requirement: スコープ別の保存先解決
`xcom-rs` は `--global` により保存先の基準ディレクトリを切り替えなければならない（MUST）。

#### Scenario: グローバルスコープでの保存先解決
- **Given** 利用者が `xcom-rs install-skills --agent opencode --global --yes` を実行したとき
- **When** CLI がインストール処理を完了するとき
- **Then** `~/.agents/skills/<skill-name>/SKILL.md` が生成され、`~/.config/opencode/skills/<skill-name>` が作成される

### Requirement: 非対話モードとJSON出力
`xcom-rs` は `--non-interactive` または `--yes` 指定時に対話プロンプトを省略し、`--json` 指定時に結果を機械可読で出力しなければならない（MUST）。

#### Scenario: JSON 出力での結果取得
- **Given** 利用者が `xcom-rs install-skills --json --non-interactive` を実行したとき
- **When** CLI がインストール結果を出力するとき
- **Then** 共通 Envelope の `data.installed_skills[]` に `name` `canonical_path` `target_paths` が含まれる

### Requirement: `--skill` 指定時のスキル解決
`xcom-rs` は `--skill <skill-name>` が指定された場合、`skills/<skill-name>/SKILL.md` を解決して当該スキルのみをインストール対象にしなければならない（MUST）。

#### Scenario: `--skill` 指定で単一スキルを解決
- **Given** 利用者が `xcom-rs install-skills --skill sample-skill --non-interactive --json` を実行したとき
- **When** CLI がインストール対象を決定するとき
- **Then** `skills/sample-skill/SKILL.md` が解決され、共通 Envelope の `data.installed_skills[]` には当該スキルのみが含まれる

