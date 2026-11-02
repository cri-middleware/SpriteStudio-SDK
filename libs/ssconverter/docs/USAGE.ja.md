[**English**](./USAGE.md) | [**日本語**](./USAGE.ja.md)

# libssconverter 組み込みガイド

`libssconverter` は、SpriteStudio のプロジェクトファイル (`.sspj`) をランタイム向けの最適化バイナリ形式 (`.ssab`) に変換するための C 互換ライブラリです。自作のゲームエンジンやツール、開発用エディタにインプロセスでのコンバート機能を組み込む際に利用します。

## 1. 準備

### ライブラリのリンク

`libssconverter` は Rust で実装されており、C-API を通じて利用します。プラットフォームに応じた静的ライブラリまたは動的ライブラリをプロジェクトにリンクしてください。

- **Windows**: `ssconverter.lib`（共有ライブラリとして使う場合は `ssconverter.dll` も）
- **macOS/Linux**: `libssconverter.a`（共有ライブラリとして使う場合は `libssconverter.dylib` / `.so` も）

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
// 第4引数は embedded 出力（テクスチャを .ssab に内包）の有効化。従来どおり（テクスチャを隣接 PNG として
// 参照）にする場合は false を指定します。
// 第5引数はログ用コールバック（不要な場合は nullptr）
ss_converter_convert(ctx, sspj_path, output_dir, /*embedded=*/false, [](const char* msg) {
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
        const char* err_msg = nullptr;
        uintptr_t len = 0;
        ss_converter_get_error(ctx, &err_msg, &len);
        if (err_msg) {
            printf("エラー: %.*s\n", (int)len, err_msg);
            // ss_converter_get_error で得られたバッファは、同一 Context で次の変換を開始するか ss_converter_destroy を呼ぶまで有効です
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

> **注意**: `ss_converter_abort` を呼び出した直後に `ss_converter_destroy` を呼び出さないでください。ワーカースレッドが完全に停止するまで待ち、必ず `ss_converter_is_finished(ctx) == true` になってから破棄してください。詳細は後述のエディタ連携のベストプラクティスを参照してください。

## 4. エディタへの組み込みとベストプラクティス

ゲームエンジン（Godot や Unity など）のエディタ拡張として `libssconverter` を組み込む場合、単一ファイルの変換だけでなく、以下の対応が求められます。

- 複数ファイル・ディレクトリの再帰的な一括バッチコンバート
- エディタのメインループでのポーリングとプログレス表示
- ユーザーキャンセル時の安全な中断（UAF を防ぐための `abort` と `destroy` の連携）

これらの詳細な実装パターンや、Godot / Unity 向けの具体的なコード例については、Player 移植ガイドの以下を参照してください。

👉 **[エディタ統合](../../ssruntime/docs/50_features/60_editor_integration.ja.md)**

## 5. 注意事項

- **メモリ管理**: `ss_converter_create` で作成したコンテキストは、必ず `ss_converter_destroy` で破棄してください。
- **スレッドセーフ**: 1 つの `Context` が同時に実行できるジョブは 1 つだけです。内部スレッドでジョブが実行されている間は、同一の `Context` に対して新たな `ss_converter_convert` や `ss_converter_scan_dir` を呼び出さないでください。
- **パスの形式**: ライブラリに渡すパスは、絶対パスを使用することを推奨します。
