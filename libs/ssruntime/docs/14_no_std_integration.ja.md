# SpriteStudio-SDK: `no_std` 統合ガイド

`libs/ssruntime` は、ファイルシステムやスレッド機構を持たない組み込み環境、ゲームコンソール (PS5 / Nintendo Switch など)、あるいは独自のメモリアロケータを採用しているゲームエンジン向けに **`#![no_std]` を完全サポート** しています。

SDK の `scripts/release-*.sh` が出力する **すべての配布バイナリ (iOS / Android / macOS / Linux / Windows) は `no_std` ビルド** で統一されており、プラットフォーム間で bit-identical な計算結果を保証します。`std` フィーチャは IDE 補完やテスト実行など開発時の DX のためにのみ用意されています。

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
cargo build -p ssruntime --release --features "libc_alloc,panic-handler"

# Wasm (ブラウザ / WebWorker / wasm-bindgen) 用 no_std ビルド
cargo build -p ssruntime --target wasm32-unknown-unknown --release --no-default-features --features "web"

# Pure Wasm (JS 非依存・WASI や組み込み Wasm エンジン向け) 用 no_std ビルド
cargo build -p ssruntime --target wasm32-unknown-unknown --release --no-default-features --features "wasm_alloc,panic-handler"

# Rust ネイティブで rlib として組み込む (ホストが allocator/panic_handler を提供する場合)
cargo build -p ssruntime --release --no-default-features --features "ffi"
```

---

## 2. C/C++ での利用 (コンソール向け / `no_std`)

C/C++ 製のゲームエンジンから FFI を経由して `ssruntime` を呼び出す際、Rust の標準ライブラリ (`std`) によるオーバーヘッドや未サポート API エラーを避けることができます。

「`std` は使えないが、C 言語の標準ライブラリ (`malloc` / `free`) は利用できる」という環境（一般的なコンソールゲーム機やモバイル環境）向けにビルドするには:

```bash
cargo build -p ssruntime --release --features "libc_alloc,panic-handler"
```

これにより `no_std` を維持しつつ、メモリアロケーションを C 言語の `malloc` にフォールバックさせ、パニックハンドラを注入した静的ライブラリ (`.a` または `.lib`) が生成されます。パニック発生時は `libc::abort()` が呼ばれて安全にプロセスが終了 (またはデバッガにブレーク) します。

WASI や JS に依存しない組み込み Pure Wasm 環境に `ssruntime` をコンパイルする場合は、以下のように `wasm_alloc` を指定します。内部で `dlmalloc` がリンクされます。

```bash
cargo build -p ssruntime --target wasm32-unknown-unknown --release --no-default-features --features "wasm_alloc,panic-handler"
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

「C の標準の `malloc` も使わず、ゲームエンジンが提供する専用のメモリ確保関数 (例: Unreal Engine の `FMalloc` やメモリプール) のみを使いたい」という場合、SDK 内部のアロケータには依存せず、ユーザー側で FFI 用のラッパークレート (wrapper crate) を作成して独自の `#[global_allocator]` を注入するのがベストプラクティスです。

### 実装ステップ

1. お客様のプロジェクト内に新しいラッパークレート (例: `ssruntime-ffi-wrapper`) を作成します。
2. このラッパークレートで `ssruntime` を `default-features = false` に設定して取り込みます。
3. ラッパークレート内で FFI バインディングと独自の `#[global_allocator]` を以下のように定義します。

```rust
// ssruntime-ffi-wrapper/src/lib.rs
#![no_std]

use core::alloc::{GlobalAlloc, Layout};

// ssruntime を公開する
pub use ssruntime::*;

// 外部の C++ アロケータ関数をリンク (例)
extern "C" {
    fn my_engine_malloc(size: usize) -> *mut u8;
    fn my_engine_free(ptr: *mut u8);
}

struct EngineAllocator;

#[global_allocator]
static ALLOCATOR: EngineAllocator = EngineAllocator;

unsafe impl GlobalAlloc for EngineAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        my_engine_malloc(layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        my_engine_free(ptr)
    }
}

// Panic Handler の定義
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {} // ログ出力や独自のエラーハンドリングを追加
}
```

このラッパークレートを `cdylib` または `staticlib` としてビルドすることで、外部 C++ エンジンのメモリ管理機構と完全に一体化した `ssruntime` アセットができあがります。

---

## 6. `std` と `no_std` ビルドの挙動の違い

`ssruntime` は同じソースコードから `std` / `no_std` の両方でビルドできますが、**いくつかの低レベル箇所で実装が切り替わります**。Player を実装する際は、本番に出る `no_std` 側の挙動を基準に考えてください。

### 6.1 数学関数

`libs/ssruntime/src/core/math.rs` で、`sin` / `cos` / `tan` / `atan2` / `abs` / `round` / `floor` / `fmod` / `sqrt` の実装が切り替わります。

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
| 実装 | Rust の System allocator | `LibcAlloc` (本ファイル `lib.rs:18-82`) |
| 内部 | `libc::malloc/free` (macOS/Linux/iOS/Android) / MSVCRT (Windows) | 同上 |

主要プラットフォームでは **どちらも libc 経由のため実質同一**です。

### 6.4 テストカバレッジ

- `cargo test` は **`std` フィーチャでのみ走ります** (`#[cfg(all(test, feature = "std"))]` で gate)
- CI の `no_std` ジョブは **コンパイル可否のみ確認** (実行テストは行いません)

`libm` と `std::f32` で結果が乖離する稀な領域は単体テストで検出されないため、最終的な数値検証は **実機の no_std ビルドで確認** することを推奨します。

---

## 7. Player 実装側の注意点

### 7.1 リンクしている `ssruntime` は `no_std` (libm 経由) であることを意識する

`scripts/release-*.sh` が出す全配布物は `no_std` です。Player が想定すべき前提:

- 三角関数は `libm` クレートの結果 (プラットフォーム libm ではない)
- パニック時のログは出ない
- グローバルアロケータは libc malloc 直呼び

これにより **iOS / Android / Windows / macOS / Linux 間でアニメ出力は bit-identical** になります。これは SDK のセールスポイントの 1 つ ([AGENTS.md](../../../AGENTS.md) "100% bit-identical results across all platforms") です。

### 7.2 IDE でのデバッグ結果は "参考値"

`.vscode/settings.json` および `.idea/cargo.xml` で `ssruntime/std` を有効化しているため、**IDE 上で実行される単体テストや `cargo run` の結果はプラットフォーム libm 経由** です。

- `cargo test --features std` で出した数値は本番 (no_std) と ULP 単位で違うかもしれない
- 最終的な数値検証は **release ビルドの no_std `staticlib` で実機確認** する

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

[目次に戻る](./README.ja.md)
