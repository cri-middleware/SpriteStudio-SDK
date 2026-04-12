# Contributing to SpriteStudio7-SDK

Thank you for your interest in contributing to SpriteStudio7-SDK! We welcome all kinds of contributions, including bug reports, feature requests, documentation improvements, and code contributions.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Environment](#development-environment)
- [Coding Standards](#coding-standards)

## Code of Conduct
Please refer to [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Reporting Bugs
If you find a bug, please open an issue and include:
- A clear, descriptive title.
- Steps to reproduce the issue.
- Your environment (OS, Rust version, target platform).
- Expected vs. actual behavior.

### Suggesting Enhancements
If you have an idea, please open an issue to discuss it before starting work.

### Pull Requests
1. Fork the repository and create your branch from `main`.
2. Ensure your code follows the existing style and conventions.
3. Add tests for any new features or bug fixes.
4. Ensure all tests pass (`cargo test`).
5. Update documentation if necessary (READMEs, Doc comments).
6. Submit a Pull Request with a clear description of the changes.

## Development Environment

### Prerequisites
- [Rustup](https://rustup.rs/) (Stable channel)
- [FlatBuffers compiler](https://google.github.io/flatbuffers/) (Required if modifying `.fbs` files)
- [cbindgen](https://github.com/mozilla/cbindgen) (For C-API header generation)

### Build and Test
```bash
# Build the entire workspace
cargo build --release

# Run all tests
cargo test

# Run the converter CLI for verification
cargo run -p ssconverter-cli -- <path_to_sspj>
```

## Coding Standards

### Rust Guidelines
- Use `rustfmt` to format your code: `cargo fmt`.
- Check for common mistakes with `clippy`: `cargo clippy`.
- Follow [Idiomatic Rust](https://github.com/mre/idiomatic-rust) practices.

### FFI and C-API
Since this SDK provides C-compatible interfaces (`ssruntime`, `ssconverter`), please pay attention to:
- **Binary Compatibility**: Avoid breaking changes to the C-API unless absolutely necessary.
- **Header Generation**: Ensure the C++ headers are regenerated and verified if you modify FFI functions.
- **Memory Management**: Clearly document who owns the memory across the FFI boundary.

---
By contributing to this project, you agree that your contributions will be licensed under the project's [LICENSE.md](./LICENSE.md).

<br>
<br>

---

# SpriteStudio7-SDK への貢献について

SpriteStudio7-SDK にご関心をお寄せいただき、ありがとうございます！ バグ報告、機能提案、ドキュメントの改善、コードの提供など、あらゆる形での貢献を歓迎します。

## 目次
- [行動規範](#行動規範-1)
- [貢献する方法](#貢献する方法-1)
- [開発環境のセットアップ](#開発環境のセットアップ-1)
- [コーディング規約](#コーディング規約-1)

## 行動規範
[CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) を参照してください。

## 貢献する方法

### バグの報告
バグを発見した場合は、Issue を作成し、以下の情報を含めてください：
- 簡潔で分かりやすいタイトル
- 再現手順
- 実行環境（OS、Rust バージョン、ターゲットプラットフォーム）
- 期待される動作と実際の動作

### 機能の提案
SDK を改善するアイデアがある場合は、開発を始める前に Issue を作成して内容を議論しましょう。

### プルリクエスト
1. リポジトリをフォークし、`main` ブランチから作業用ブランチを作成してください。
2. 既存のスタイルや規約に従ってコードを記述してください。
3. 新しい機能やバグ修正には、対応するテストを追加してください。
4. すべてのテストがパスすることを確認してください (`cargo test`)。
5. 必要に応じてドキュメント (README やコメント) を更新してください。
6. 変更内容を明確に記載したプルリクエストを送信してください。

## 開発環境のセットアップ

### 前提条件
- [Rustup](https://rustup.rs/) (Stable チャンネル)
- [FlatBuffers コンパイラ](https://google.github.io/flatbuffers/) (`.fbs` ファイルを変更する場合に必要)
- [cbindgen](https://github.com/mozilla/cbindgen) (C-API ヘッダー生成に必要)

### ビルドとテスト
```bash
# ワークスペース全体のビルド
cargo build --release

# すべてのテストを実行
cargo test

# コンバーター CLI を実行して動作確認
cargo run -p ssconverter-cli -- <path_to_sspj>
```

## コーディング規約

### Rust のガイドライン
- `rustfmt` を使用してフォーマットを統一してください: `cargo fmt`
- `clippy` を使用して一般的なミスをチェックしてください: `cargo clippy`
- [慣用的な Rust](https://github.com/mre/idiomatic-rust) (Idiomatic Rust) のプラクティスに従ってください。

### FFI と C-API
この SDK は C 互換インターフェースを提供しているため、以下の点に注意してください：
- **バイナリ互換性**: 避けられない場合を除き、C-API の破壊的変更は避けてください。
- **ヘッダー生成**: FFI 関数を変更した場合は、C++ ヘッダーが正しく更新・検証されていることを確認してください。
- **メモリ管理**: FFI 境界を越えてデータをやり取りする場合、メモリの所有権を明確にドキュメント化してください。

---
本プロジェクトへの貢献は、プロジェクトの [LICENSE.md](./LICENSE.md) に同意したものとみなされます。
