# refresh-readme-quickstart 設計メモ

## 方針
- ドキュメントのみ更新し、実装や仕様は変更しない。
- 参照源は `src/cli.rs` のコマンド定義と `docs/examples.md` の既存例とする。

## Quick Start構成
1. インストール
2. 認証（import例）
3. doctorで状態確認
4. 代表操作（tweets create など）

## 変更境界
- READMEとdocs/examplesのみに限定する
