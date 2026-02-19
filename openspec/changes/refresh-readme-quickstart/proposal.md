# refresh-readme-quickstart 提案

## なぜ
- READMEのQuick Startが最小限で、初回成功体験が伝わりにくい。
- 実装済みのコマンド群（thread/search/timeline/media/bookmarks/doctor）がREADMEのFeaturesや例に反映されていない。

## 何を変えるか
- Quick Startを「インストール→認証→doctor→代表操作」の最短導線に更新する。
- Features一覧を現在のCLI定義に合わせて拡充する。
- docs/examplesにdoctor・thread・timeline・search・media・bookmarksの例を追加する。

### 対象範囲
- `README.md` のQuick Start/Features/Examplesセクション
- `docs/examples.md` の使用例追加

### 非対象
- CLIの挙動変更
- 既存API仕様の変更

### 成果物
- Quick Startの更新と誤差のない機能一覧
- 主要コマンドの実行例

### 受け入れ条件
- READMEの記載が `src/cli.rs` のコマンド定義と一致する
- docs/examplesに新規例が追加され、既存例と整合している

### リスク・留意点
- 追加コマンドが増え続ける前提で、READMEの継続的同期が必要
