> **日本語** | [English](./10_no_std_integration.md)

# no_std 組み込み環境への統合

`libs/ssruntime` は、ファイルシステムやスレッド機構を持たない組み込み環境、ゲームコンソール (PS5 / Nintendo Switch など)、あるいは独自のメモリアロケータを採用しているゲームエンジン向けに **`#![no_std]` を完全サポート** しています。

SDK が公式に配布する **すべてのバイナリ (iOS / Android / macOS / Linux / Windows) は `no_std` ビルド** で統一されており、プラットフォーム間で bit-identical な計算結果を保証します。`std` フィーチャは IDE 補完やテスト実行など開発時の DX のためにのみ用意されています。

---

## 1. ビルド構成の概要

| Feature | デフォルト | 役割 |
|---|---|---|
| `ffi` | ✅ | C-API エクスポート |
| `libc_alloc` | — | `libc::malloc/free` ベースの `#[global_allocator]` |
| `wasm_alloc` | — | Pure Wasm 用 `dlmalloc` ベースの `#[global_allocator]` |
| `panic-handler` | — | `#[panic_handler]` を提供 (パニック時に `libc::abort` または Wasm `unreachable`) |
| `std` | — | `flatbuffers` / `ssab` を std モードに、テスト実行用 `ssconverter` を依存に追加 |
| `web` | — | `wasm-bindgen` バインディング (`wasm-bindgen` 自体の `std` フィーチャを有効化します) |

> [!NOTE]
> `web` フィーチャを有効にすると、ブラウザでの動作に必要な `wasm-bindgen` がリンクされ、それに伴い Rust の `std` がアロケータやパニックハンドラを提供します。しかし、`ssruntime` 内部の計算（数学関数など）は引き続き `libm` クレートを使用するため、他のプラットフォームとの決定性（Bit-identical）は維持されます。

各環境向けの代表的なコマンド:

```bash
# モバイル / コンソール / デスクトップ用 no_std staticlib (推奨)
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"

# Wasm (ブラウザ / WebWorker / wasm-bindgen) 用 no_std ビルド
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "web"

# Pure Wasm (JS 非依存・WASI や組み込み Wasm エンジン向け) 用 no_std ビルド
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "wasm_alloc,panic-handler"

# Rust ネイティブで rlib として組み込む (ホストが allocator/panic_handler を提供する場合)
cargo build -p ssruntime-ffi --release --no-default-features --features "ffi"
```

---

## 2. C/C++ での利用 (コンソール向け / `no_std`)

C/C++ 製のゲームエンジンから FFI を経由して `ssruntime` を呼び出す際、Rust の標準ライブラリ (`std`) によるオーバーヘッドや未サポート API エラーを避けることができます。

「`std` は使えないが、C 言語の標準ライブラリ (`malloc` / `free`) は利用できる」という環境（一般的なコンソールゲーム機やモバイル環境）向けにビルドするには:

```bash
cargo build -p ssruntime-ffi --release --features "libc_alloc,panic-handler"
```

これにより `no_std` を維持しつつ、メモリアロケーションを C 言語の `malloc` にフォールバックさせ、パニックハンドラを注入した静的ライブラリ (`.a` または `.lib`) が生成されます。パニック発生時は `libc::abort()` が呼ばれて安全にプロセスが終了 (またはデバッガにブレーク) します。

WASI や JS に依存しない組み込み Pure Wasm 環境に `ssruntime` をコンパイルする場合は、以下のように `wasm_alloc` を指定します。内部で `dlmalloc` がリンクされます。

```bash
cargo build -p ssruntime-ffi --target wasm32-unknown-unknown --release --no-default-features --features "wasm_alloc,panic-handler"
```

---

## 3. デバッグログ出力の移譲 (Log Callback)

`no_std` 環境では、Rust の標準出力 (`println!`, `eprintln!`) が利用できません。
そのため、`ssruntime` 内部からのエラーや警告ログを受け取るには、初期化時にログコールバックを登録する必要があります。

```cpp
#include "runtime/ssruntime.h"

// ログを受け取るコールバック関数
void my_log_callback(int level, const char* message) {
    // 例: printf("libssruntime [%d]: %s\n", level, message);
    // ゲームエンジンのログ出力関数 (UE_LOG, Debug.Log, print_line など) に渡す
}

// 初期化の最初にコールバックを登録する
ss_runtime_set_log_callback(my_log_callback);
```

これにより、Rust 内部でパースエラーや警告が発生した際に、ホスト側 (C/C++ や C#) にメッセージが転送されます。

---

## 4. Rust ネイティブでの利用

自作の Rust 製ゲームエンジン (`no_std` アプリケーション) から `ssruntime` を呼び出す場合、アロケータやパニックハンドラの手配は上位の実行バイナリ (Executable) が担うのが一般的です。

`Cargo.toml` に以下のように追加し、`default-features = false` にすることで、`libc_alloc` を無効化した純粋な `no_std` ライブラリ (`rlib`) として利用可能です。

```toml
[dependencies]
ssruntime = { version = "7.0", default-features = false, features = ["ffi"] }
```

> [!NOTE]
> 必須機能である `Vec` などは内部的に `alloc` クレートを利用して動作します。最終的なターゲットバイナリで何らかの `#[global_allocator]` と `#[panic_handler]` が提供されている必要があります。

---

## 5. 完全な独自のカスタムアロケータの利用 (Unreal Engine 等)

「C の標準の `malloc` も使わず、ゲームエンジンが提供する専用のメモリ確保関数 (例: Unreal Engine の `FMalloc` やメモリプール) のみを使いたい」という場合、ラッパークレートを作成することなく、`custom_alloc` フィーチャを有効化するだけで簡単にカスタムアロケータを C/C++ 側から注入できます。

### 実装ステップ

1. `ssruntime` をビルドする際に、`libc_alloc` の代わりに `custom_alloc` を指定します。配布物として使うのは **staticlib (`libssruntime.a` / `.lib`)** なので、staticlib を明示してビルドします。

```bash
cargo rustc -p ssruntime-ffi --release --features "custom_alloc,panic-handler" --crate-type staticlib
```

> [!IMPORTANT]
> `custom_alloc` では `ss_custom_*` シンボルや `memcpy` 等の C ランタイム関数は **ssruntime 側では未解決のまま**にしておき、最終的にエンジン側とリンクするときに解決します。そのため `crate-type` に `cdylib` を含む素の `cargo build -p ssruntime-ffi` を実行すると、未解決シンボルを許さない macOS / Windows (MSVC) のリンカが `cdylib` のリンク段階で失敗します。上記のように `--crate-type staticlib` を明示してビルドしてください（成果物は `target/release/libssruntime.a`）。

2. リンクする C/C++ のプロジェクト（Unreal Engine など）側で、以下の 3 つの `extern "C"` 関数を定義します。これらに加えて、`memcpy` / `memset` / `memmove` / `memcmp` / `strlen` といった C ランタイム関数も最終リンク時に必要になりますが、通常はリンクするゲームエンジンの CRT が提供します（ベアメタル環境では別途用意してください）。

```cpp
#include <stddef.h>
#include <stdlib.h>
#if defined(_MSC_VER)
#include <malloc.h> // _aligned_malloc / _aligned_free
#endif

extern "C" {

// 1. メモリ確保
//    返すポインタは必ず `align` バイト境界に整列している必要があります
//    (`align` は常に 2 の冪)。素の malloc は max_align_t (通常 8〜16B) しか
//    保証しないため、SIMD 型などの過剰アラインメント要求で未定義動作になります。
//    ssruntime はアラインメント補正を行わないので、必ず整列確保を使ってください。
void* ss_custom_alloc(size_t size, size_t align) {
    // 例: Unreal Engine の場合
    // return FMemory::Malloc(size, align);
#if defined(_MSC_VER)
    return _aligned_malloc(size, align);
#else
    // posix_memalign は align が sizeof(void*) の倍数である必要があるため引き上げる
    if (align < sizeof(void*)) {
        align = sizeof(void*);
    }
    void* ptr = NULL;
    if (posix_memalign(&ptr, align, size) != 0) {
        return NULL; // 確保失敗は NULL を返す (Rust 側が検知する)
    }
    return ptr;
#endif
}

// 2. メモリ解放
//    確保時と同じ size / align が渡されます。解放時に size を必要とするプール
//    アロケータはここで利用できます (malloc 系は無視して構いません)。
//    ※確保関数と対になる解放関数を使うこと。特に MSVC は _aligned_free が必須で、
//      _aligned_malloc したメモリを free すると未定義動作になります。
void ss_custom_free(void* ptr, size_t size, size_t align) {
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

// 3. パニックハンドラ (アボート処理)
// ※戻り値なし (noreturn) として定義する必要があります
//   (リターンすると Rust 側の `-> !` 契約に違反し未定義動作になります)
#if defined(_MSC_VER)
__declspec(noreturn)
#elif defined(__GNUC__) || defined(__clang__)
__attribute__((noreturn))
#endif
void ss_custom_abort() {
    // 例: UE_LOG で致命的エラーを出力してクラッシュさせる
    // UE_LOG(LogTemp, Fatal, TEXT("ssruntime panic"));
    abort();
}

}
```

これらの関数を C/C++ 側で定義し、生成された `libssruntime.a` (または `.lib`) をリンクするだけで、Rust 側のソースコードを一切触ることなく、シームレスにゲームエンジンのアロケータと統合されます。

> [!NOTE]
> `ss_custom_abort` が呼ばれるのは Rust 側で **panic** が発生したときだけです。`ss_custom_alloc` が `NULL` を返した場合 (メモリ確保失敗) は Rust の alloc エラーハンドラ経由で即座に abort するため、`ss_custom_abort` は経由しません。確保失敗時に独自処理を行いたい場合は `ss_custom_alloc` 内でハンドリングしてください。
>
> なお `custom_alloc` と `libc_alloc` / `wasm_alloc` / `emscripten_alloc` は `#[global_allocator]` が競合するため同時に有効化できません (`compile_error!` で検出されます)。

---

## 6. `std` と `no_std` ビルドの挙動の違い

`ssruntime` は同じソースコードから `std` / `no_std` の両方でビルドできますが、**いくつかの低レベル箇所で実装が切り替わります**。Player を実装する際は、本番に出る `no_std` 側の挙動を基準に考えてください。

### 6.1 数学関数

`std` / `no_std` で、`sin` / `cos` / `tan` / `atan2` / `abs` / `round` / `floor` / `fmod` / `sqrt` の実装が切り替わります。

| | std | no_std |
|---|---|---|
| 実装 | `f.sin()`, `f.cos()`, ... (`std::f32` メソッド) | `libm::sinf`, `libm::cosf`, ... (`libm` クレート、純 Rust 実装) |
| 中身 | プラットフォーム libm (macOS の Accelerate / Linux の glibc 等) | Sun fdlibm の Rust 移植 |

トリゴノメトリ系は **ULP 単位で異なる結果**を返す可能性があります。`sqrt` / `round` / `floor` / `abs` はハードウェア命令で実装されることが多く、両者で完全一致することがほとんどです。

実用上の影響:
- 人の目で見たアニメ再生結果は同じ
- ハッシュ比較などビット単位の比較を行うと差分が出る可能性
- **配布バイナリは全プラットフォームが no_std (libm 統一) なので、プラットフォーム間では bit-identical**

### 6.2 パニック処理

| | std | no_std + `panic-handler` |
|---|---|---|
| パニック時 | stderr にメッセージとバックトレースを出力 → abort | `libc::abort()` を即呼ぶ (メッセージ出力なし) |

ワークスペースの `Cargo.toml` で `panic = "abort"` が設定されているため、unwind は両方とも発生しません。`no_std` ビルドでは **パニック理由がログに残らない** ため、Player 側で SIGABRT を捕まえても原因特定は困難です。

### 6.3 グローバルアロケータ

| | std | no_std + `libc_alloc` |
|---|---|---|
| 実装 | Rust の System allocator | `LibcAlloc` (`libc_alloc` フィーチャが提供) |
| 内部 | `libc::malloc/free` (macOS/Linux/iOS/Android) / MSVCRT (Windows) | 同上 |

主要プラットフォームでは **どちらも libc 経由のため実質同一**です。

---

## 7. Player 実装側の注意点

### 7.1 リンクしている `ssruntime` は `no_std` (libm 経由) であることを意識する

SDK が公式配布するバイナリはすべて `no_std` ビルドです。Player が想定すべき前提:

- 三角関数は `libm` クレートの結果 (プラットフォーム libm ではない)
- パニック時のログは出ない
- グローバルアロケータは libc malloc 直呼び

これにより **iOS / Android / Windows / macOS / Linux 間でアニメ出力は bit-identical**（全プラットフォームで完全一致）になります。

### 7.2 `std` ビルドの数値は "参考値"

開発時の DX 向けに `std` フィーチャ（IDE 補完・テスト実行用）でビルドした場合、数学関数はプラットフォーム libm 経由になり、本番の `no_std` 配布物とは三角関数の結果が ULP 単位で異なる可能性があります。

- `std` ビルドで確認した数値は本番 (no_std) と ULP 単位で違うかもしれない
- 最終的な数値検証は **`no_std` ビルドで実機確認** する

### 7.3 パニックを Player レイヤで握り潰さない

`ssruntime` は `panic = "abort"` で no_std ビルドでもメッセージなしに abort します。Player レイヤで `signal(SIGABRT)` 等をトラップして無視すると **どこで死んだか永久に追えません**。

- 渡す引数 (`Context::new()` 後の `bind_resource`、`setup_animation` のフレーム範囲など) は **Player 側で事前に validate** する
- `ssruntime` の API は `Result<_, SsError>` を返すものが多いので、エラーは silently ignore せず Player のロガーに転送する
- セクション 3 の `ss_runtime_set_log_callback` を必ず登録しておく

### 7.4 浮動小数決定性に依存するゲームロジック

ネットワーク同期 / リプレイ / ロックステップ系で「アニメ結果のハッシュ比較」を行う設計の場合:

- **同一 SDK バージョン同士なら全プラットフォームで bit-identical** (libm 統一のおかげ)
- **SDK バージョンを跨ぐと libm のアップデートで差分が出る可能性がある**
- ハッシュ比較を行うなら、SDK バージョンも合わせて固定すること

### 7.5 アロケータ統計のズレ

Player が独自アロケータ (Godot / Unreal の `MemoryArena`、Unity の `Native Memory` など) を使っていても、`libc_alloc` ビルドの `ssruntime` は **libc malloc を直接呼ぶ** ため Player 側のアロケータ統計には乗りません。

- メモリ計測でズレを許容する、または
- セクション 5 のラッパークレートパターンで Player のアロケータをフックする

の 2 択になります。

### 7.6 Static lib 同士のリンクで出る duplicate symbol 警告

Godot custom module や GDExtension のように `libssruntime.a` と `libssconverter.a` を同じ最終バイナリに静的リンクすると、リンカが以下のような警告を出します:

```
ld: warning: duplicate symbol '__rust_alloc' in libssruntime.a / libssconverter.a
ld: warning: duplicate symbol 'rust_eh_personality' in ...
ld: warning: duplicate symbol 'rust_begin_unwind' in ...
```

**実害はありません**:
- `panic = "abort"` のため `rust_eh_personality` / `rust_begin_unwind` は実質 no-op
- アロケータシムは std System 側も `LibcAlloc` 側も libc malloc 経由で機能等価
- ld は最初に見つけた実装を採用してリンク自体は成立

警告が気になる場合は `ssruntime` + `ssconverter` を依存に持つラッパークレートを作って 1 本の staticlib にまとめると消せますが、通常は許容で問題ありません。

---

[目次に戻る](../README.ja.md)
