[**English**](./README.md) | [**日本語**](./README.ja.md)

# libssab

SpriteStudio 7 のアニメーションバイナリデータ (`.ssab`, `.ssqb`) を読み書きするための Rust ライブラリです。
Google FlatBuffers を使用してシリアライズ/デシリアライズを行います。

# 対応フォーマット

このライブラリは以下の2つのフォーマット定義 (`fbs`) をサポートしています。

- **SSAB ([ssab.fbs](./fbs/ssab.fbs))**: アニメーションデータ (`SsAnimeBinary`)
- **SSQB ([ssqb.fbs](./fbs/ssqb.fbs))**: シーケンスデータ (`SsSequenceBinary`)

# インストール

`Cargo.toml` に以下のように記述して使用します。

```toml
[dependencies]
ssab = { path = "libs/ssab" }
```

# Features (機能フラグ)

用途に応じて `features` を切り替える必要があります。

## `default` (Runtime)
デフォルトの状態です。データの読み込み（デシリアライズ）に最適化されています。
ゲームランタイムやプレイヤーなど、データの再生のみを行う場合に適しています。

- **特徴**: Read-only アクセス、ゼロコピー読み込み。
```toml
[dependencies]
ssab = { path = "libs/ssab" }
```

## `converter`
データの生成や編集を行うツール向けのモードです。
FlatBuffers の Object API (`--gen-object-api`) が有効になり、Rust の構造体としてデータを操作し、バイナリへの書き出しが可能になります。

- **特徴**: Read/Write アクセス、Object API (T構造体) の利用可能。
```toml
[dependencies]
ssab = { path = "libs/ssab", features = ["converter"] }
```

# 使用例

## データの読み込み (Runtime)
```rust
use std::fs;
use ssab::ssab::ss::format::get_root_as_ss_anime_binary;

fn main() {
    let bytes = fs::read("test.ssab").unwrap();

    // バイナリからルートオブジェクトを取得
    let anime = get_root_as_ss_anime_binary(&bytes).unwrap();
    
    println!("Anime Name: {}", anime.name().unwrap_or("unknown"));
    println!("Total Frames: {}", anime.total_frame());
}
```

## データの作成/書き出し (Converter)

`converter` feature を有効にする必要があります。

```rust
#[cfg(feature = "converter")]
fn create_ssab() {
use flatbuffers::FlatBufferBuilder;
use ssab::ssab::ss::format::{SsAnimeBinary, SsAnimeBinaryT};

    // Object API (T構造体) を使用してデータを構築
    let mut anime_obj = SsAnimeBinaryT::default();
    anime_obj.name = Some("MyAnimation".to_string());
    anime_obj.fps = 60;
    anime_obj.total_frame = 100;

    // FlatBufferBuilder でシリアライズ
    let mut builder = FlatBufferBuilder::new();
    let offset = anime_obj.pack(&mut builder);
    builder.finish(offset, Some(SsAnimeBinary::IDENTIFIER));

    let buf = builder.finished_data();
    // buf をファイルに書き出すなど...
}
```

# 開発・コード生成

`fbs` ファイルを変更した場合、Rust コードを再生成する必要があります。
再生成には `flatc` コマンド (Google FlatBuffers Compiler) がインストールされている必要があります。

## Windows (PowerShell)

```powershell
./scripts/generate.ps1
```

## macOS / Linux (Bash)

```bash
./scripts/generate.sh
```

このスクリプトは以下の処理を行います：

1. `fbs` フォルダ内の定義ファイルをコンパイル。
2. `runtime` 用と `converter` 用のソースコードをそれぞれ `src/runtime` と `src/converter` に生成。
3. `converter` 用のコードには Object API 生成オプションなどを付与。


