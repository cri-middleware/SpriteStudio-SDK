# SpriteStudio7-SDK: `no_std` 統合ガイド

`libs/ssruntime` は、ファイルシステムやスレッド機構を持たない組み込み環境、ゲームコンソール (PS5 / Nintendo Switch など)、あるいは独自のメモリアロケータを採用しているゲームエンジン向けに **`#![no_std]` を完全サポートし、デフォルト設定としています**。

このドキュメントでは、`no_std` 環境下で本 SDK (特に `ssruntime`) をプロジェクトに統合するためのアプローチと、必要な設定を解説します。

---

## 1. C/C++ での利用 (コンソール向け / デフォルトの `libc_alloc`)

C/C++ 製のゲームエンジンから FFI を経由して `ssruntime` を呼び出す際、Rust の標準ライブラリ（`std`）によるオーバーヘッドや未サポートAPIエラーを避けることができます。

「`std` は使えないが、C言語の標準ライブラリ（`malloc` / `free`）は利用できる」という環境（一般的なコンソールゲーム機やモバイル環境）向けに、デフォルトで **`libc_alloc` 機能** が有効になっています。

### ビルド手順

以下のコマンドを実行することで、`no_std` を維持しつつ、メモリアロケーションを C 言語の `malloc` にフォールバックさせた静的ライブラリ (`.a` または `.lib`) または動的ライブラリ (`.so`, `.dll`, `.dylib`) を生成できます。

ビルド時には、C-API として独立して機能させるため、ライブラリ内にパニック時の処理を含める **`custom-panic-handler`** フィーチャーを有効化してください。

```bash
cargo build -p ssruntime --release --features "custom-panic-handler"
```

これにより、`no_std` なバイナリサイズの大幅な削減と、ネイティブ環境での安定したメモリ確保が両立できます。
パニック発生時は `libc::abort()` が呼ばれて安全にプロセスが終了（またはデバッガにブレーク）するよう調整されています。

---

## 2. デバッグログ出力の移譲 (Log Callback)

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

## 3. Rust ネイティブでの利用

自作の Rust 製ゲームエンジン（`no_std` アプリケーション）から `ssruntime` を呼び出す場合、アロケータやパニックハンドラの手配は上位の実行バイナリ (Executable) が担うのが一般的です。

`Cargo.toml` に以下のように追加し、`default-features = false` にすることで、`libc_alloc` を無効化した純粋な `no_std` ライブラリ (`rlib`) として利用可能です。

```toml
[dependencies]
ssruntime = { version = "7.0", default-features = false }
```

> [!NOTE]
> 必須機能である `Vec` などは内部的に `alloc` クレートを利用して動作します。最終的なターゲットバイナリで何らかの `#[global_allocator]` と `#[panic_handler]` が提供されている必要があります。

---

## 4. 完全な独自のカスタムアロケータの利用 (Unreal Engine 等)

「Cの標準の `malloc` も使わず、ゲームエンジンが提供する専用のメモリ確保関数 (例: Unreal Engine の `FMalloc` やメモリプール) のみを使いたい」という場合、SDK 内部のアロケータには依存せず、ユーザー側で FFI 用のラッパークレート (wrapper crate) を作成して独自の `#[global_allocator]` を注入するのがベストプラクティスです。

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
