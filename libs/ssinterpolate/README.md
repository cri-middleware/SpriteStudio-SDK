[**日本語**](./README.ja.md) | [**English**](./README.md)

# ssinterpolate

A platform-agnostic math library for SpriteStudio animation interpolations.

## Features

- **Generic Implementation**: Provides a unified math logic for both `f32` and `f64` precision using Rust generics and traits.
- **Pure Math Logic**: Zero-dependency library focused solely on interpolation algorithms (Linear, Hermite, Bezier, and various Easing functions).
- **FFI Support**: Optional C-API (FFI) wrappers for integration with C/C++ runtimes (enabled via the `ffi` feature).
- **Independent**: Does not depend on `ssab` or other SpriteStudio data structures, making it highly reusable.

## Usage (Rust)

```rust
use ssinterpolate::interpolate::*;

let start = 0.0f32;
let end = 100.0f32;
let rate = 0.5f32;

// Linear interpolation (f32)
let value = linear(start, end, rate);

// Generic support for f64
let value_f64 = linear(0.0f64, 100.0f64, 0.5f64);

// Easing
let eased_rate = rate_ease_in(rate, 2.0);
let eased_value = linear(start, end, eased_rate);
```

## Features Flags

- `ffi`: (Off by default) Enables `extern "C"` wrappers for C/C++ interoperability.
