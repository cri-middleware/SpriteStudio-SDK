[**English**](./README.md) | [**日本語**](./README.ja.md)

# ssconverter-cli

SpriteStudio 7 のプロジェクトファイル (`.sspj`) を解析し、ランタイムに最適化されたバイナリファイル (`.ssab`, `.ssqb`) に変換するコマンドラインツールです。

[libssconverter](../libs/ssconverter) のラッパーとして機能し、コマンドラインから手軽に変換機能を利用できます。

# 使い方

`ssconverter-cli` には、**シングルモード** と **バッチモード** の2つの動作モードがあります。

## シングルモード (Single Mode)

単一の `.sspj` プロジェクトファイルを変換します。
このモードでは、出力ディレクトリの指定や、ファイル監視（Watcher）機能が利用可能です。

```bash
# 単一のプロジェクトを変換
$ ssconverter-cli project.sspj

# 出力ディレクトリを指定して変換
$ ssconverter-cli project.sspj -o ./output_folder
```

### オプション (シングルモード)
- `-e`, `--embedded`: 参照されているアセット（画像など）を生成される `.ssab` ファイル内に直接埋め込みます。
- `-o`, `--out-dir <directory>`: 生成ファイルの出力先ディレクトリを指定します。
- `-w`, `--watcher`: プロジェクトディレクトリを監視し、ファイルが変更されたら自動的に再変換を行います。
- `-h`, `--help`: ヘルプを表示します。

## バッチモード (Batch Mode)

複数の `.sspj` プロジェクトをまとめて変換します。
複数のファイルパス、またはディレクトリを指定することができます（ディレクトリ内の `.sspj` は再帰的に検索されます）。
このモードでは `--out-dir` オプションは無視され、出力ファイルはソースファイルの横（またはデフォルトのディレクトリ）に生成されます。

```bash
# 特定の複数ファイルを変換
$ ssconverter-cli project1.sspj project2.sspj

# 指定ディレクトリ以下のすべての sspj を変換
$ ssconverter-cli ./assets/projects ./extra_assets
```

### オプション (バッチモード)
- `-e`, `--embedded`: アセットを `.ssab` ファイル内に埋め込みます。
- `-h`, `--help`: ヘルプを表示します。

_注意: バッチモードでは `--out-dir` および `--watcher` オプションは利用できません。_

# 使用例

**1. 基本的な変換**
`project.sspj` と同じ階層に `project_ssab/` フォルダが生成されます。

```bash
$ ssconverter-cli my_game/project.sspj
```

**2. 監視モード (Watch Mode)**
プロセスを常駐させ、SpriteStudio で保存などの変更が行われるたびに自動で再変換します。

```bash
$ ssconverter-cli my_game/project.sspj --watch
```

**3. アセット埋め込み**
テクスチャなどの依存ファイルをすべて含んだ単一のバイナリを生成します。配布や管理を単純化したい場合に便利です。

```bash
$ ssconverter-cli my_game/project.sspj --embedded
```

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
