[**English**](./README.md) | [**日本語**](./README.ja.md)

# libssconverter

SpriteStudio 7 のプロジェクトファイル (`.sspj`) を、ランタイムに最適化されたバイナリ形式 (`.ssab`, `.ssqb`) に変換するためのライブラリです。

`sspj` クレートを使用してプロジェクトを解析し、`ssab` クレートを使用して FlatBuffers バイナリを生成します。

# 主な機能

- **プロジェクト変換**: `.sspj` (プロジェクト) を `.ssab` (アニメーション) に、`.ssqe` (シーケンス) を `.ssqb` に変換します。
- **最適化**: FlatBuffers のシリアライズ時に、座標、色、文字列などのデータ構造を重複排除 (Deduplication) します。これにより、バイナリサイズを大幅に削減（複雑なアニメーションでは 50% 以上削減されることもあります）し、実行時のキャッシュ効率を向上させます。
- **埋め込みモード**: 参照されているアセット（画像など）をバイナリ内に直接埋め込むオプションをサポートしています。
- **C-API / FFI**: C++ など他の言語から利用するための C 互換インターフェースを提供します。

# インストール

`Cargo.toml` に以下のように記述して使用します。

```toml
[dependencies]
ssconverter = { path = "libs/ssconverter" }
```

# 使い方 (Rust)

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
        output: None, // デフォルトの出力先 (例: project_ssab/) を使用
    };

    // 変換を実行してファイルを保存 (ログ出力用コールバックを指定)
    convert_sspj_to_ssab_files(sspj_path, &config, &|msg| println!("{}", msg))?;
    
    Ok(())
}
```

# FFI (C/C++)

このライブラリは、コンバータの作成や進捗監視を行うための C API を公開しています。
詳細な組み込み方法については、[libssconverter 組み込みガイド](./docs/USAGE.ja.md) を参照してください。

ヘッダファイルは `cbindgen` によって `target/ssconverter.h` に生成されます。

## C++ API 使用例

```cpp
#include "ssconverter.h"

void convert() {
    ss_converter_context* ctx = ss_converter_create();
    
    // バックグラウンドスレッドで変換を開始
    // ログが不要な場合はコールバックに nullptr を渡します
    ss_converter_convert(ctx, "path/to/project.sspj", "output/dir", nullptr);
    
    // 完了を待機
    while (ss_converter_is_running(ctx)) {
        // sleep or update progress
    }
    
    // 結果の確認
    CConverterError result = ss_converter_get_result(ctx);
    if (result != CConverterError::Success) {
        char* err_msg;
        size_t len;
        ss_converter_get_error(ctx, &err_msg, &len);
        printf("Error: %.*s\n", (int)len, err_msg);
    }
    
    ss_converter_destroy(ctx);
}
```

# ビルド機能 (Features)

- **`ffi`** (default): C-API および静的/動的ライブラリの生成を有効にします。
- **`web`**: WASM サポートを有効にします (実験的)。

# アーキテクチャ

- **`converter`**: `sspj` のデータ構造を `ssab` の FlatBuffers スキーマにマッピングするコアロジック。
- **`cache`**: FlatBuffers オブジェクトのキャッシュと重複排除ロジックを実装し、ファイルサイズを削減します。
- **`ffi`**: C-API のラッパー関数群。

