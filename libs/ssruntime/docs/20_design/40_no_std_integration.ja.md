> **日本語** | [English](./40_no_std_integration.md)

# no_std 環境への統合

Brain は `#![no_std]` で動きます。ファイルシステムやスレッドを持たない組み込み環境、ゲームコンソール、独自のメモリアロケータを使うエンジンにも組み込めます。SDK が配布するネイティブのバイナリ（iOS / Android / macOS / Linux / Windows）はすべて `no_std` でビルドしています。このページは、自分でビルドする場合と、エンジンのアロケータを使いたい場合に読んでください。

---

## 1. ビルド構成

| feature | 既定 | 役割 |
|---|---|---|
| `ffi` | ✅ | C-API を公開する |
| `web` | — | `wasm-bindgen` のバインディング。`wasm-bindgen` が `std` を有効にする |
| `libc_alloc` | — | libc の `malloc` / `free` を使うグローバルアロケータ（モバイル・デスクトップ） |
| `wasm_alloc` | — | `dlmalloc` を使うグローバルアロケータ（JavaScript に依存しない WebAssembly） |
| `emscripten_alloc` | — | Emscripten の `malloc` / `free` を使うグローバルアロケータ |
| `custom_alloc` | — | ホストが用意する `ss_custom_alloc` / `ss_custom_free` / `ss_custom_abort` を使う（§4） |
| `panic-handler` | — | パニックハンドラ。パニックしたときに `abort` する（WebAssembly では `unreachable`） |
| `std` | — | 開発時のためのもので、リリースのビルドでは使わない |

グローバルアロケータの feature（`libc_alloc` / `wasm_alloc` / `emscripten_alloc` / `custom_alloc`）は、1 つだけを選んでください。2 つ以上を有効にするとコンパイルエラーになります。

`web` を有効にすると Rust の `std` がアロケータとパニックハンドラを提供しますが、Brain の数学関数は引き続き `libm` を使うので、他のプラットフォームと同じ結果になります（§5-1）。

```bash
# モバイル / コンソール / デスクトップ向けの静的ライブラリ
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"

# ブラウザ向け（wasm-bindgen）
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "web"

# JavaScript に依存しない WebAssembly（WASI や組み込みの WebAssembly エンジン）
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "ffi,wasm_alloc,panic-handler"

# Rust のクレートとして組み込む（アロケータとパニックハンドラはホストが用意する）
cargo build -p ssruntime --release
```

---

## 2. C / C++ から使う

`std` は使えないが C の標準ライブラリ（`malloc` / `free`）は使える環境（一般的なコンソールやモバイル）では、次のようにビルドします。

```bash
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"
```

メモリの確保は C の `malloc` で行い、パニックしたときは `abort()` でプロセスを終了（またはデバッガで停止）する静的ライブラリ（`.a` / `.lib`）ができます。

---

## 3. Rust のクレートとして使う

Rust で書かれた `no_std` のエンジンから使う場合、アロケータとパニックハンドラは、最終的な実行バイナリが用意するのが一般的です。`Cargo.toml` に次のように書きます。既定で有効な feature は無いので、アロケータも C-API も持たない `no_std` のライブラリになります。

```toml
[dependencies]
ssruntime = "7.0.0-beta.1"
```

> [!NOTE]
> Brain は `alloc` クレートの `Vec` などを使います。最終的なバイナリに `#[global_allocator]` と `#[panic_handler]` があることが必要です。

---

## 4. エンジンのアロケータを使う（`custom_alloc`）

C の `malloc` も使わず、エンジンのメモリ確保関数（Unreal Engine の `FMalloc` やメモリプールなど）だけを使いたい場合は、`custom_alloc` でビルドし、確保と解放の関数を C / C++ 側で定義します。Rust 側のコードを書く必要はありません。

### 4-1. ビルドする

配布に使うのは静的ライブラリなので、`--crate-type staticlib` を指定してビルドします。

```bash
cargo rustc -p ssruntime-ffi --release --features "custom_alloc,panic-handler" --crate-type staticlib
```

> [!IMPORTANT]
> `custom_alloc` では、`ss_custom_*` と `memcpy` などの C ランタイム関数を未解決のまま残し、エンジンとリンクするときに解決します。そのため、`crate-type` に `cdylib` を含む通常の `cargo build -p ssruntime-ffi` は、未解決のシンボルを許さない macOS と Windows（MSVC）のリンカで失敗します。上のように `--crate-type staticlib` を指定してください（成果物は `target/release/libssruntime.a`）。

### 4-2. 関数を定義する

リンクする C / C++ のプロジェクトで、次の 3 つの `extern "C"` 関数を定義します。`memcpy` / `memset` / `memmove` / `memcmp` / `strlen` などの C ランタイム関数も最終的なリンクで必要ですが、通常はエンジンの CRT が提供します（ベアメタル環境では別に用意してください）。

```cpp
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#if defined(_MSC_VER)
#include <malloc.h> // _aligned_malloc / _aligned_free
#endif

extern "C" {

// 1. メモリの確保
//    返すポインタは `align` バイト境界に揃っていなければなりません（`align` は常に 2 の冪で、8 以上。
//    Player は FrameData の構造体をその場で読むので、Brain は 8 未満を要求しません）。
//    素の malloc は max_align_t（通常 8〜16 バイト）までしか保証しないので、
//    それより大きな境界を要求されると未定義動作になります。Brain は境界を補正しません。
uint8_t* ss_custom_alloc(uintptr_t size, uintptr_t align) {
    // 例: Unreal Engine の場合
    // return FMemory::Malloc(size, align);
#if defined(_MSC_VER)
    return (uint8_t*)_aligned_malloc(size, align);
#else
    // align は 8 以上なので、posix_memalign が要求する sizeof(void*) の倍数になっている
    void* ptr = NULL;
    if (posix_memalign(&ptr, align, size) != 0) {
        return NULL; // 確保に失敗したら NULL を返す（Brain が検知する）
    }
    return (uint8_t*)ptr;
#endif
}

// 2. メモリの解放
//    確保したときと同じ size / align が渡されます。解放に size が要るプールアロケータで使えます。
//    確保関数と対になる解放関数を使ってください。MSVC では、_aligned_malloc で確保したメモリを
//    free すると未定義動作になります。
void ss_custom_free(void* ptr, uintptr_t size, uintptr_t align) {
    (void)size;
    (void)align;
    // 例: Unreal Engine の場合
    // FMemory::Free(ptr);
#if defined(_MSC_VER)
    _aligned_free(ptr);
#else
    free(ptr);
#endif
}

// 3. パニック時の中断
//    戻ってはいけません（noreturn）。戻ると未定義動作になります。
#if defined(_MSC_VER)
__declspec(noreturn)
#elif defined(__GNUC__) || defined(__clang__)
__attribute__((noreturn))
#endif
void ss_custom_abort() {
    // 例: UE_LOG で致命的なエラーを出してクラッシュさせる
    // UE_LOG(LogTemp, Fatal, TEXT("ssruntime panic"));
    abort();
}

}
```

この 3 つを定義して `libssruntime.a`（または `.lib`）をリンクすれば、Brain のメモリ確保はすべてエンジンのアロケータを通ります。

> [!NOTE]
> `ss_custom_abort` が呼ばれるのは、Brain でパニックが起きたときだけです。`ss_custom_alloc` が `NULL` を返した場合（確保の失敗）は、Rust のアロケーションエラーの処理ですぐに abort し、`ss_custom_abort` は通りません。確保の失敗に独自の処理をしたい場合は、`ss_custom_alloc` の中で行ってください。

---

## 5. `std` と `no_std` の違い

Brain は同じソースから `std` と `no_std` の両方でビルドできますが、いくつかの低レベルの処理の実装が切り替わります。**配布しているのは `no_std` のビルドなので、Player はそちらの挙動を前提にしてください。**

### 5-1. 数学関数

`sin` / `cos` / `tan` / `atan2` / `abs` / `round` / `floor` / `fmod` / `sqrt` の実装が切り替わります。

| | std | no_std |
|---|---|---|
| 実装 | `f32` のメソッド（`f.sin()` など） | `libm` クレート（`libm::sinf` など。Sun fdlibm の Rust 移植） |
| 中身 | プラットフォームの libm（macOS の Accelerate、Linux の glibc など） | すべてのプラットフォームで同じ Rust の実装 |

三角関数は、両者で ULP 単位の差が出ることがあります。`sqrt` / `round` / `floor` / `abs` はハードウェア命令になることが多く、ほぼ一致します。見た目には同じですが、ビット単位で比べると差が出ることがあります。

配布バイナリはどれも `libm` を使う（WebAssembly 版を含む）ので、**同じ SDK の版どうしなら、プラットフォームが違っても結果はビット単位で一致します。** SDK の版をまたぐと、`libm` の更新で差が出ることがあります。リプレイやロックステップのようにアニメーションの結果をハッシュで比べる設計では、SDK の版も固定してください。

`std` でビルドして確かめた数値は、配布版と ULP 単位で違うことがあります。最終的な数値の確認は `no_std` のビルドで行ってください。

### 5-2. パニック

| | std | no_std + `panic-handler` |
|---|---|---|
| パニックしたとき | stderr にメッセージとバックトレースを出して abort | すぐに `abort()`（メッセージは出ない） |

どちらも `panic = "abort"` でビルドしているので、巻き戻しは起きません。配布版ではパニックの理由がログに残らないため、Player で `SIGABRT` を捕まえても原因は分かりません。**パニックを Player の層で握りつぶさないでください。** どこで止まったのかが追えなくなります。かわりに、Brain に渡す引数を Player の側で確かめ、失敗した呼び出しの理由をログに流してください（[ロードと初期化 §4](../30_core/10_initialization.ja.md#4-ログとエラー)）。

### 5-3. グローバルアロケータ

| | std | no_std + `libc_alloc` |
|---|---|---|
| 実装 | Rust の System アロケータ | `libc_alloc` が提供する `LibcAlloc` |
| 中身 | libc の `malloc` / `free`（Windows は MSVCRT） | 同じ |

主要なプラットフォームでは、どちらも libc を通るので同じです。

`libc_alloc` でビルドした Brain は libc の `malloc` を直接呼ぶので、Player がエンジン独自のアロケータ（Unity の Native Memory など）を使っていても、Brain のメモリはそのアロケータの統計に載りません。統計に載せたい場合は `custom_alloc`（§4）でエンジンのアロケータを渡してください。

---
[目次に戻る](../README.ja.md)
