[**English**](./README.md) | [**日本語**](./README.ja.md)

# libssab

SpriteStudio のアニメーションバイナリデータ (`.ssab`, `.ssqb`) を読み書きするための Rust ライブラリです。
Google FlatBuffers を使用してシリアライズ/デシリアライズを行います。

# 対応フォーマット

このライブラリは以下の2つのフォーマット定義 (`fbs`) をサポートしています。詳細な設計仕様については、[**スキーマドキュメント**](./SCHEMA.ja.md) を参照してください。

- **SSAB ([ssab.fbs](./fbs/ssab.fbs))**: アニメーションデータ (`SsAnimeBinary`)
- **SSQB ([ssqb.fbs](./fbs/ssqb.fbs))**: シーケンスデータ (`SsSequenceBinary`)

# インストール

`Cargo.toml` に以下のように記述して使用します。

```toml
[dependencies]
ssab = "7.0.0-beta.1"
```

# Features (機能フラグ)

用途に応じて `features` を切り替える必要があります。

## `default` (Runtime)
デフォルトでは `std` feature (`flatbuffers/std`) が有効になります。データの読み込み（デシリアライズ）に最適化されており、標準ライブラリ環境でデータの再生・読み取りのみを行う場合に適しています。

- **特徴**: Read-only アクセス、ゼロコピー読み込み。
```toml
[dependencies]
ssab = "7.0.0-beta.1"
```

## `no_std` サポート
`libssab` は `no_std` 環境（`core` + `alloc`）と互換性があります。`no_std` で使用する場合は、デフォルト機能を無効にして `std` 依存を外します：

```toml
[dependencies]
ssab = { version = "7.0.0-beta.1", default-features = false }
```

## `converter`
データの生成や編集を行うツール向けのモードです（`std` を含みます）。
FlatBuffers の Object API (`--gen-object-api`) が有効になり、Rust の構造体としてデータを操作し、バイナリへの書き出しが可能になります。

- **特徴**: Read/Write アクセス、Object API (T構造体) の利用可能。
```toml
[dependencies]
ssab = { version = "7.0.0-beta.1", features = ["converter"] }
```

# 使用例

## データの読み込み (Runtime)
```rust
use std::fs;
use ssab::ssab::ss::format::root_as_ss_anime_binary;

fn main() {
    let bytes = fs::read("test.ssab").unwrap();

    // バイナリからルートオブジェクトを取得。`root_as_*` はバッファを検証します
    // （検証を省く `_unchecked` 版もあります）。
    let binary = root_as_ss_anime_binary(&bytes).unwrap();

    println!("Name: {}", binary.name());
    for anime in binary.animations() {
        println!("  {} ({} frames @ {} fps)", anime.name(), anime.total_frame(), anime.fps());
    }
}
```

## データの作成/書き出し (Converter)

`converter` feature を有効にする必要があります。

```rust
#[cfg(feature = "converter")]
fn create_ssab() {
    use flatbuffers::FlatBufferBuilder;
    use ssab::ssab::ss::format::{AnimationDataT, SsAnimeBinaryT, finish_ss_anime_binary_buffer};

    // Object API (T構造体) を使用してデータを構築。T構造体は `#[non_exhaustive]`
    // なので、`default()` から始めてフィールドを代入します。
    // fps などアニメーション単位の情報は `AnimationDataT` にあります。
    let mut anime = AnimationDataT::default();
    anime.name = "MyAnimation".to_string();
    anime.fps = 60;
    anime.total_frame = 100;

    // `SsAnimeBinaryT` がルートで、パーツとアニメーション一覧を保持します。
    let mut binary = SsAnimeBinaryT::default();
    binary.name = "MyProject".to_string();
    binary.animations = vec![anime];

    // FlatBufferBuilder でシリアライズ。ヘルパーが "SSAB" ファイル識別子を
    // 書き込みます（`root_as_ss_anime_binary` はこれを検査します）。
    let mut builder = FlatBufferBuilder::new();
    let offset = binary.pack(&mut builder);
    finish_ss_anime_binary_buffer(&mut builder, offset);

    let buf = builder.finished_data();
    // buf をファイルに書き出すなど...
}
```

# 開発・コード生成 (fbsファイルの更新)

`fbs/` 内の `.fbs` ファイルを変更した場合は、`./scripts/generate.sh`（Windows では `./scripts/generate.ps1`）を実行して `src/runtime/` と `src/converter/` の Rust コードを再生成してください。使う `flatc` のバージョンは [CONTRIBUTING.md](../../CONTRIBUTING.md#前提条件) にあります。


