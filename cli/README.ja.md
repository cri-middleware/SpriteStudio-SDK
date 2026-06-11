[**English**](./README.md) | [**日本語**](./README.ja.md)

# ssconverter-cli

[SpriteStudio7-SDK](https://github.com/SpriteStudio/SpriteStudio7-SDK/blob/main/README.ja.md) のワークフローにおけるビルド時のツールです。SpriteStudio 7 のプロジェクトファイル (`.sspj` と、そこから参照される `.ssae` / `.ssee` / `.ssqe` / `.ssce`) を解析し、SDK が再生するランタイムバイナリ — `.ssab`（アニメーション）と `.ssqb`（シーケンスパック） — を生成します。

本 CLI は [libssconverter](https://github.com/SpriteStudio/SpriteStudio7-SDK/tree/main/libs/ssconverter) のラッパーであり、SDK のリリース成果物のビルドに使われているのと同じ変換パイプラインを公開しています。ランタイム側は [libssruntime](https://github.com/SpriteStudio/SpriteStudio7-SDK/tree/main/libs/ssruntime) を参照してください。

# 使い方

`ssconverter-cli` には、**シングルモード** と **バッチモード** の2つの動作モードがあります。

## シングルモード (Single Mode)

単一の `.sspj` プロジェクトファイルを変換します。
このモードでは、出力ディレクトリの指定が可能です。

```bash
# 単一のプロジェクトを変換
$ ssconverter-cli project.sspj

# 出力ディレクトリを指定して変換
$ ssconverter-cli project.sspj -o ./output_folder
```

### オプション (シングルモード)
- `-e`, `--embedded`: 参照されているアセット（画像など）を生成される `.ssab` ファイル内に直接埋め込みます。
- `-o`, `--out-dir <directory>`: 生成ファイルの出力先ディレクトリを指定します。
- `-h`, `--help`: ヘルプを表示します。

## バッチモード (Batch Mode)

複数の `.sspj` プロジェクトをまとめて変換します。
複数の file パス、またはディレクトリを指定することができます（ディレクトリ内の `.sspj` は再帰的に検索されます）。
このモードでは `--out-dir` オプションは無視され、出力は各 `.sspj` の隣に `<name>_ssab/` として生成されます。

```bash
# 特定の複数ファイルを変換
$ ssconverter-cli project1.sspj project2.sspj

# 指定ディレクトリ以下のすべての sspj を変換
$ ssconverter-cli ./assets/projects ./extra_assets
```

### オプション (バッチモード)
- `-e`, `--embedded`: アセットを `.ssab` ファイル内に埋め込みます。
- `-h`, `--help`: ヘルプを表示します。

_注意: バッチモードでは `--out-dir` オプションは利用できません。_

# インストール

## ビルド済みバイナリ（推奨）

[GitHub Releases](https://github.com/SpriteStudio/SpriteStudio7-SDK/releases) ページから **Windows**、**macOS**、**Linux** 向けのリリースをダウンロードしてください。ビルド環境は不要です。

## ソースからビルド

Rust ツールチェイン（[rustup](https://rustup.rs/)）が必要です。リポジトリのルートから:

```bash
$ cargo install --path cli
```

# 使用例

**1. 基本的な変換**
`project.sspj` と同じ階層に `project_ssab/` フォルダが生成されます。

```bash
$ ssconverter-cli my_game/project.sspj
```

**2. アセット埋め込み**
テクスチャなどの依存ファイルをすべて含んだ単一のバイナリを生成します。配布や管理を単純化したい場合に便利です。

```bash
$ ssconverter-cli my_game/project.sspj --embedded
```

# 出力

`.sspj` を変換するたびに、ソースファイルの隣にディレクトリが生成されます（シングルモードで `--out-dir` を指定した場合を除く）:

| モード | 出力ディレクトリ | 中身 |
|---|---|---|
| デフォルト | `<name>_ssab/` | `.ssab`（アニメーションごとに 1 つ）、`.ssqb`（`.ssqe` が参照されている場合のシーケンスパック）、参照されているテクスチャのコピー |
| `--embedded` | `<name>_emb_ssab/` | 同上ですが、参照アセットは各 `.ssab` 内に埋め込まれます（テクスチャの別ファイルは生成されません） |

# 次のステップ

`.ssab` ファイルが生成できたら、[libssruntime](https://github.com/SpriteStudio/SpriteStudio7-SDK/tree/main/libs/ssruntime) を通じてゲームエンジンに組み込みます。実装手順は [Player 移植ガイド](https://github.com/SpriteStudio/SpriteStudio7-SDK/blob/main/libs/ssruntime/docs/README.ja.md) を参照してください。

# 開発者向け
## ビルド

Rust 環境が必要です。

```bash
$ cargo build --release
# バイナリは ./target/release/ssconverter-cli に生成されます
```

## テスト

`src/main.rs` に定義された統合テストを実行します。

```bash
$ cargo test
```
