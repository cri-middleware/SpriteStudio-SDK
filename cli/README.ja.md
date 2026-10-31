[**English**](./README.md) | [**日本語**](./README.ja.md)

# ssconverter-cli

[SpriteStudio-SDK](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/README.ja.md) のワークフローにおけるビルド時のツールです。SpriteStudio のプロジェクトファイル (`.sspj` と、そこから参照される `.ssae` / `.ssee` / `.ssqe` / `.ssce`) を解析し、SDK が再生するランタイムバイナリ — `.ssab`（アニメーション）と `.ssqb`（シーケンスパック） — を生成します。

本 CLI は [libssconverter](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssconverter) のラッパーで、各 Player に組み込まれたインポーターと同じ変換を行います。ランタイム側は [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssruntime) を参照してください。

# インストール

## ビルド済みバイナリ（推奨）

[GitHub Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) ページから **Windows**、**macOS**、**Linux** 向けのリリースをダウンロードしてください。ビルド環境は不要です。

各リリースにはプラットフォームごとの zip が固定の名前で入っているため、最新版は常に同じ URL で取得できます。

| プラットフォーム | アーカイブ |
|---|---|
| macOS（Apple silicon・Intel） | `ssconverter-cli-macos.zip` |
| Windows x64 | `ssconverter-cli-windows.zip` |
| Windows ARM64 | `ssconverter-cli-windows-arm64.zip` |
| Linux x64 | `ssconverter-cli-linux.zip` |
| Linux ARM64 | `ssconverter-cli-linux-aarch64.zip` |

```bash
$ curl -fsSLO https://github.com/cri-middleware/SpriteStudio-SDK/releases/latest/download/ssconverter-cli-macos.zip
$ unzip ssconverter-cli-macos.zip -d ssconverter-cli
$ ./ssconverter-cli/ssconverter-cli --version
```

版を固定するには `latest/download` を `download/<tag>` に置き換えてください。同じリリースの `SHA256SUMS` で zip を検証できます。

## ソースからビルド

Rust ツールチェイン（[rustup](https://rustup.rs/)）の **1.99 以降** が必要です。リポジトリのルートから:

```bash
$ cargo install --path cli
```

# 使い方

`ssconverter-cli` には、**シングルモード** と **バッチモード** の2つの動作モードがあります。

## シングルモード (Single Mode)

単一の `.sspj` プロジェクトファイルを変換します。このモードでは、出力ディレクトリの指定が可能です。

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

複数の `.sspj` プロジェクトをまとめて変換します。複数の file パス、またはディレクトリを指定することができます（ディレクトリ内の `.sspj` は再帰的に検索されます）。このモードでは `--out-dir` オプションは無視され、出力は各 `.sspj` の隣に `<name>/` として生成されます。

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

## 終了ステータス

どちらのモードでも、指定したパスが存在しないときや、プロジェクトの変換に失敗したときは、0 以外の終了ステータスで終わります。該当するパスとプロジェクトはすべて標準エラーに出力され、変換できるプロジェクトはそのまま変換されます。

# 使用例

**1. 基本的な変換**

`project.sspj` と同じ階層に `project/` フォルダが生成されます。

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
| デフォルト | `<name>/` | `.ssab`（アニメーションごとに 1 つ）、`.ssqb`（`.ssqe` が参照されている場合のシーケンスパック）、参照されているテクスチャのコピー |
| `--embedded` | `<name>_emb/` | 同上ですが、参照アセットは各 `.ssab` 内に埋め込まれます（テクスチャの別ファイルは生成されません） |

# 次のステップ

`.ssab` ファイルが生成できたら、お使いの Player に渡します。置き場所はその Player のドキュメントにあります。ゲームエンジンに自分で組み込む場合は [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssruntime) を通じて読み込みます。実装手順は [Player 移植ガイド](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssruntime/docs/README.ja.md) を参照してください。

# 開発者向け
## ビルド

Rust 環境が必要です。

```bash
$ cargo build --release
# バイナリは ./target/release/ssconverter-cli に生成されます
```

## テスト

`tests/` のテストは、ビルドした CLI を実行して終了ステータスを確かめます。リポジトリのルートで次を実行します。

```bash
$ cargo test -p ssconverter-cli
```
