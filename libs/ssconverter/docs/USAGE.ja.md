# libssconverter 組み込みガイド

`libssconverter` は、SpriteStudio 7 のプロジェクトファイル (`.sspj`) をランタイム向けの最適化バイナリ形式 (`.ssab`) に変換するための C 互換ライブラリです。
自作のゲームエンジンやツール、開発用エディタにインプロセスでのコンバート機能を組み込む際に利用します。

## 1. 準備

### ライブラリのリンク

`libssconverter` は Rust で実装されており、C-API を通じて利用します。
プラットフォームに応じた静的ライブラリまたは動的ライブラリをプロジェクトにリンクしてください。

- **Windows**: `ssconverter.lib` (および `ssconverter.dll` if shared)
- **macOS/Linux**: `libssconverter.a` (および `libssconverter.dylib`/`.so` if shared)

また、各プラットフォームで必要なシステムライブラリ（pthread, dl 等）を適宜追加してください。

### ヘッダのインクルード

`ssconverter.h` をプロジェクトに含め、ソースコードからインクルードします。

```cpp
#include "ssconverter.h"
```

## 2. 基本的な利用フロー

`libssconverter` の API は、バックグラウンドスレッドで変換処理を実行し、呼び出し側はポーリングによって完了を監視する非同期的な設計になっています。

### コンテキストの作成

変換処理を管理するための `Context` オブジェクトを作成します。

```cpp
Context* ctx = ss_converter_create();
```

### 変換の開始

`ss_converter_convert` を呼び出すと、内部でワーカースレッドが開始され、変換処理が実行されます。この関数自体は即座に復帰します。

```cpp
const char* sspj_path = "C:/Assets/Character.sspj";
const char* output_dir = "C:/Project/Assets/SSAB/";

// 変換を開始
// 第4引数はログ用コールバック（不要な場合は nullptr）
ss_converter_convert(ctx, sspj_path, output_dir, [](const char* msg) {
    printf("SSConverter: %s\n", msg);
});
```

### 進行状況の監視

エディタのメインループやフレーム更新（Update）の中で、`ss_converter_is_finished` を呼び出して完了を確認します。

```cpp
void on_update() {
    if (ctx && ss_converter_is_finished(ctx)) {
        // 変換が完了した
        on_conversion_complete(ctx);
    }
}
```

### 結果の取得とクリーンアップ

変換が完了したら、結果を確認してコンテキストを破棄します。

```cpp
void on_conversion_complete(Context* ctx) {
    CConverterError result = ss_converter_get_result(ctx);
    
    if (result == CConverterError::Success) {
        printf("変換成功！\n");
    } else {
        // エラー詳細の取得
        char* err_msg = nullptr;
        uintptr_t len = 0;
        ss_converter_get_error(ctx, &err_msg, &len);
        if (err_msg) {
            printf("エラー: %.*s\n", (int)len, err_msg);
            // ss_converter_get_error で得られたメモリは ss_converter_destroy で解放されます
        }
    }

    // コンテキストの破棄（必須）
    ss_converter_destroy(ctx);
}
```

## 3. 高度な機能

### 変換の中断

時間のかかる処理をユーザーがキャンセルした場合などは `ss_converter_abort` を呼び出します。

```cpp
ss_converter_abort(ctx);
```

## 4. 実装例: Godot Plugin での利用

`SSPlayerForGodot` では、エディタへのファイルドロップ時に `libssconverter` を呼び出し、インプロセスで `.ssab` を生成しています。

```cpp
// gd_ss_import_dock.cpp の抜粋

void GdSsImportControl::_notification(int p_what) {
    if (p_what == NOTIFICATION_PROCESS) {
        if (is_importing) {
            bool all_finished = true;
            for (void* ctx : import_contexts) {
                if (!ss_converter_is_finished((Context*)ctx)) {
                    all_finished = false;
                    break;
                }
            }

            if (all_finished) {
                // 終了処理とリソースのスキャン
                is_importing = false;
                set_process(false);
                // ... cleanup contexts ...
            }
        }
    }
}
```

このように、複数のファイルを同時に並列変換することも可能です（`libssconverter` 内部でも `rayon` による並列処理が行われますが、コンテキストを複数作ることで複数のプロジェクトを並行して扱うことができます）。

## 5. 注意事項

- **メモリ管理**: `ss_converter_create` で作成したコンテキストは、必ず `ss_converter_destroy` で破棄してください。
- **スレッドセーフ**: 変換処理は内部スレッドで行われますが、同一の `Context` に対して複数の `ss_converter_convert` を同時に呼び出さないでください。
- **パスの形式**: ライブラリに渡すパスは、絶対パスを使用することを推奨します。
