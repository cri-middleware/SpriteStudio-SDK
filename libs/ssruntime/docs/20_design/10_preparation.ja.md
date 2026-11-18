> **日本語** | [English](./10_preparation.md)

# 事前確認とプロジェクトの準備

Player を書き始める前に、対象のエンジンで Brain を使えるかを確かめ、ビルドが通るだけの空のプロジェクトを用意します。

## 1. 対応できるかを確かめる

Brain は Rust で書かれた C-API のライブラリで、フレームごとの出力は FlatBuffers のバッファです。対象のエンジンと言語について、次の 3 点を確かめてください。

1. **Rust がそのプラットフォームをビルドできるか。** 対象の CPU アーキテクチャとプラットフォームが [Rust のプラットフォームサポート](https://doc.rust-lang.org/beta/rustc/platform-support.html) にあるかを確認します。SDK のリリースにそのプラットフォーム向けのライブラリが無い場合は、自分でビルドします（[no_std 環境への統合 §1](./40_no_std_integration.ja.md#1-ビルド構成)）。
2. **エンジンの言語から C の関数（または WebAssembly）を呼べるか。** エンジンがネイティブライブラリか WebAssembly を読み込めること、その言語から関数をバインドして呼べることを確認します。
3. **FlatBuffers のコード生成がその言語に対応しているか。** [FlatBuffers のサポート言語](https://flatbuffers.dev/support/) を確認します。

## 2. プロジェクトを用意する

1. 対象のエンジンと言語で、ビルドできる空のプロジェクトを作ります。
2. `flatc` を入手し、SDK のスキーマ（`libs/ssab/fbs/` と `libs/ssruntime/fbs/`）から、使う言語の読み取りコードを生成します。生成したコードと FlatBuffers のランタイムライブラリをプロジェクトに取り込みます。

    ```bash
    # C++ の場合
    flatc --cpp -o ./src/gen/ libs/ssab/fbs/*.fbs
    flatc --cpp -o ./src/gen/ libs/ssruntime/fbs/*.fbs
    ```

3. `libssruntime` の静的ライブラリ（`.lib` / `.a`）か動的ライブラリ（`.dll` / `.so` / `.dylib`）をプロジェクトに置き、リンクします。C / C++ から使う場合は、ヘッダ `ssruntime.h` をインクルードパスに加えます。

## 3. テスト用の ssab を用意する

1. `ssconverter-cli` を [GitHub Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) から入手します（[ssconverter-cli の README](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/cli/README.ja.md)）。
2. SDK のリポジトリで `ssconverter-cli tests/overall/overall.sspj` を実行し、ssab に変換します。このプロジェクトには、機能ごとのアニメーションが揃っています（[目次の「実装の進め方」](../README.ja.md#1-実装の進め方)）。
3. 変換した ssab（`tests/overall/overall/` に出力されます）をプロジェクトのアセットとして置きます。


---
[目次に戻る](../README.ja.md)
