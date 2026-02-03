[**日本語**](./README.ja.md) | [**English**](./README.md)

# libssconverter

A library for converting SpriteStudio 7 project files (`.sspj`) into runtime-optimized binary formats (`.ssab`, `.ssqb`).

It utilizes `sspj` to parse the project and `ssab` to generate FlatBuffers binaries.

# Features

- **Project Conversion**: Converts `.sspj` (project) to `.ssab` (animation) and `.ssqe` (sequence) to `.ssqb`.
- **Optimization**: Deduplicates data structures (like coordinates, colors, strings) during FlatBuffers serialization to minimize binary size.
- **Embedded Mode**: Optionally embeds referenced assets (images, etc.) directly into the binary.
- **File Watching**: Monitors the project directory for changes and automatically re-converts files.
- **C-API / FFI**: Provides C-compatible interfaces for integration with other languages (C++, etc.).

# Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ssconverter = { path = "libs/ssconverter" }
```

# Usage (Rust)

```rust
use std::path::Path;
use ssconverter::{convert_sspj_to_ssab_files, ConverterConfig, ConvertConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sspj_path = Path::new("path/to/project.sspj");
    
    let config = ConverterConfig {
        convert_config: ConvertConfig {
            optimize: true,   // Enable deduplication
            embedded: false,  // Do not embed textures
            ..Default::default()
        },
        out_dir: None, // Use default output directory (e.g. project_ssab/)
    };

    // Convert and save files
    convert_sspj_to_ssab_files(sspj_path, &config)?;
    
    Ok(())
}
```

# FFI (C/C++)

The library exposes a C API for creating converters and monitoring progress.
Headers are generated in `target/ssconverter.h` by `cbindgen`.

## Example C++ API Usage

```cpp
#include "ssconverter.h"

void convert() {
    ss_converter_context* ctx = ss_converter_create();
    
    // Start conversion in a background thread
    ss_converter_convert(ctx, "path/to/project.sspj", "output/dir");
    
    // Wait for finish
    while (!ss_converter_is_finished(ctx)) {
        // sleep or update progress
    }
    
    // Check result
    CConverterError result = ss_converter_get_result(ctx);
    if (result != CConverterError::Success) {
        char* err_msg;
        size_t len;
        ss_converter_get_error(ctx, &err_msg, &len);
        printf("Error: %s\n", err_msg);
    }
    
    ss_converter_destroy(ctx);
}
```


# Build Features

- **`ffi`** (default): Enables C-API and static/dynamic library generation.
- **`web`**: Enables WASM support (experimental).

# Architecture

- **`converter`**: Core logic for mapping `sspj` data structures to `ssab` FlatBuffers schemas.
- **`cache`**: Implements caching and deduplication logic for FlatBuffers objects to reduce file size.
- **`watcher`**: Directory watching functionality using `notify-debouncer-mini`.
- **`ffi`**: C-API wrapper functions.


<llm-snippet-file>README.ja.md</llm-snippet-file>
```markdown
# libssconverter

SpriteStudio 7 のプロジェクトファイル (`.sspj`) を、ランタイムに最適化されたバイナリ形式 (`.ssab`, `.ssqb`) に変換するためのライブラリです。

`sspj` クレートを使用してプロジェクトを解析し、`ssab` クレートを使用して FlatBuffers バイナリを生成します。

## 主な機能

- **プロジェクト変換**: `.sspj` (プロジェクト) を `.ssab` (アニメーション) に、`.ssqe` (シーケンス) を `.ssqb` に変換します。
- **最適化**: FlatBuffers のシリアライズ時に、座標、色、文字列などのデータ構造を重複排除 (Deduplication) し、バイナリサイズを最小化します。
- **埋め込みモード**: 参照されているアセット（画像など）をバイナリ内に直接埋め込むオプションをサポートしています。
- **ファイル監視**: プロジェクトディレクトリの変更を監視し、自動的に再変換を行う機能を備えています。
- **C-API / FFI**: C++ など他の言語から利用するための C 互換インターフェースを提供します。

## インストール

`Cargo.toml` に以下のように記述して使用します。

```toml
[dependencies]
ssconverter = { path = "libs/ssconverter" }
```

## 使い方 (Rust)

```rust
use std::path::Path;
use ssconverter::{convert_sspj_to_ssab_files, ConverterConfig, ConvertConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sspj_path = Path::new("path/to/project.sspj");
    
    let config = ConverterConfig {
        convert_config: ConvertConfig {
            optimize: true,   // 重複排除を有効化
            embedded: false,  // テクスチャを埋め込まない
            ..Default::default()
        },
        out_dir: None, // デフォルトの出力先 (例: project_ssab/) を使用
    };

    // 変換を実行してファイルを保存
    convert_sspj_to_ssab_files(sspj_path, &config)?;
    
    Ok(())
}
```

## FFI (C/C++)

このライブラリは、コンバータの作成や進捗監視を行うための C API を公開しています。
ヘッダファイルは `cbindgen` によって `target/ssconverter.h` に生成されます。

### C++ API 使用例

```cpp
#include "ssconverter.h"

void convert() {
    ss_converter_context* ctx = ss_converter_create();
    
    // バックグラウンドスレッドで変換を開始
    ss_converter_convert(ctx, "path/to/project.sspj", "output/dir");
    
    // 完了を待機
    while (!ss_converter_is_finished(ctx)) {
        // sleep or update progress
    }
    
    // 結果の確認
    CConverterError result = ss_converter_get_result(ctx);
    if (result != CConverterError::Success) {
        char* err_msg;
        size_t len;
        ss_converter_get_error(ctx, &err_msg, &len);
        printf("Error: %s\n", err_msg);
    }
    
    ss_converter_destroy(ctx);
}
```

## ビルド機能 (Features)

- **`ffi`** (default): C-API および静的/動的ライブラリの生成を有効にします。
- **`web`**: WASM サポートを有効にします (実験的)。

## アーキテクチャ

- **`converter`**: `sspj` のデータ構造を `ssab` の FlatBuffers スキーマにマッピングするコアロジック。
- **`cache`**: FlatBuffers オブジェクトのキャッシュと重複排除ロジックを実装し、ファイルサイズを削減します。
- **`watcher`**: `notify-debouncer-mini` を使用したディレクトリ監視機能。
- **`ffi`**: C-API のラッパー関数群。
```

