# 設計メモ: install-skills

## 目的
`xcom-rs` にスキル導入コマンドを追加するための最小設計。既存 CLI の非対話/JSON 出力の原則に合わせ、外部依存を持たない実装を前提とする。

## 主要な判断
### スキル探索ソース
- リポジトリ内の埋め込みディレクトリ（例: `skills/`）を探索対象とする
- スキル定義の source of truth は `skills/<skill-name>/SKILL.md` とし、このパスを基準に探索・解決する
- ネットワーク取得は扱わない

### インストール先
- canonical: `.agents/skills/<skill-name>/SKILL.md`
- `--global` 指定時は `~/.agents/skills` を canonical とする

### エージェント別ディレクトリ
- `--agent claude`: `.claude/skills/<skill-name>` にリンク/コピー
- `--agent opencode`:
  - プロジェクトスコープ: 追加リンクなし（canonical のみ）
  - グローバルスコープ: `~/.config/opencode/skills/<skill-name>` にリンク/コピー

### 失敗時のフォールバック
- シンボリックリンク作成に失敗した場合はディレクトリコピーへフォールバック
- 失敗情報は JSON 出力で明示する

## トレードオフ
- すべてを canonical に集約することで管理を簡略化できる一方、エージェント固有のパスを追加で作成する必要がある
- リンク失敗時のコピーは容量効率を下げるが、環境差分に強い
