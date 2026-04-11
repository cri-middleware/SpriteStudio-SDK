[**English**](./README.md) | [**日本語**](./README.ja.md)

# ssinterpolate

SpriteStudio のアニメーション補間計算のためのプラットフォーム非依存な数学ライブラリです。

## 特徴

- **ジェネリクスによる実装**: Rust のジェネリクスとトレイトを活用し、`f32` と `f64` の両方の精度に対して共通の数学ロジックを提供します。
- **純粋な計算ロジック**: 外部依存関係ゼロで、補間アルゴリズム（線形、Hermite、Bezier、および各種イージング関数）のみに特化しています。
- **FFI サポート**: C/C++ ランタイムとの統合のために、オプションで C-API (FFI) ラッパーを提供します（`ffi` フィーチャーで有効化）。
- **独立性**: `ssab` やその他の SpriteStudio のデータ構造に依存しないため、高い再利用性があります。

## 使い方 (Rust)

```rust
use ssinterpolate::interpolate::*;

let start = 0.0f32;
let end = 100.0f32;
let rate = 0.5f32;

// 線形補間 (f32)
let value = linear(start, end, rate);

// f64 にも対応
let value_f64 = linear(0.0f64, 100.0f64, 0.5f64);

// イージング
let eased_rate = rate_ease_in(rate, 2.0);
let eased_value = linear(start, end, eased_rate);
```

## フィーチャーフラグ

- `ffi`: (デフォルト) C/C++ 互換性のための `extern "C"` ラッパーを有効にします。
