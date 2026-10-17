# Contributing to SpriteStudio-SDK

> [!IMPORTANT]
> We are not accepting pull requests until a Contributor License Agreement (CLA) is in place. Issues remain open for bug reports and requests.

Thank you for your interest in contributing to SpriteStudio-SDK! We welcome all kinds of contributions, including bug reports, feature requests, documentation improvements, and code contributions.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Environment](#development-environment)
- [Coding Standards](#coding-standards)

## Code of Conduct
Please refer to [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Technical Constraints & Guiding Principles
To maintain the high-performance and portable nature of this SDK, all code contributions must adhere to the following core principles:

- **Strict `no_std` Compliance:** The core runtime (`libssruntime`, `libssab`) must remain `#![no_std]`. We do not allow dependencies on `std` or any platform-specific libraries that break portability.
- **Zero-Allocation in Playback Hot Path:** Dynamic memory allocation (heap allocation) is strictly prohibited during the animation playback loop. All necessary memory must be pre-allocated or provided by the caller.
- **Performance over Convenience:** We prioritize runtime execution speed and memory efficiency over "easy-to-use" helper functions if they introduce overhead.
- **SIMD-Friendly Design:** Data structures (especially in `ssab`) must be designed with SIMD (f32x4) vectorization in mind. Prefer columnar layouts and flat arrays.
- **Stateless Brain (Separation of Concerns):** The runtime (Brain) calculates states but must not have knowledge of the rendering engine (Body). Keep the interface primitive and deterministic.
- **Explicit over Implicit:** Avoid "magic" or hidden behaviors. Code should be predictable, traceable, and explicit in its intent.

### Reporting Bugs
If you find a bug, please open an issue and include:
- A clear, descriptive title.
- Steps to reproduce the issue.
- Your environment (OS, Rust version, target platform).
- Expected vs. actual behavior.

*Note: If you need to send proprietary `.sspj` projects to reproduce the bug, please do not attach them to public issues. See [SUPPORT.md](./SUPPORT.md) for how to contact us privately.*

### Suggesting Enhancements
If you have an idea, please open an issue to discuss it before starting work.

### Pull Requests
1. Fork the repository and create your branch from `develop`.
2. Ensure your code follows the existing style and conventions.
3. Add tests for any new features or bug fixes.
4. Ensure `scripts/check.sh` passes (see [Checks](#checks)).
5. Update documentation if necessary (READMEs, Doc comments).
6. Submit a Pull Request against `develop` with a clear description of the changes.

## Development Environment

### Prerequisites
- [Rustup](https://rustup.rs/) (Stable channel). The oldest Rust the workspace supports is the `rust-version` in the root `Cargo.toml`.
- [FlatBuffers compiler](https://google.github.io/flatbuffers/) (`flatc`), if you modify `.fbs` files. Use the release whose version matches the `flatbuffers` crate in the root `Cargo.toml`: the generated code calls that crate.

### Build
```bash
# Build the entire workspace
cargo build --release

# Run the converter CLI for verification
cargo run -p ssconverter-cli -- <path_to_sspj>
```

### Checks
`scripts/check.sh` (`scripts/check.ps1` in PowerShell) runs `ci.yml`'s lint job and its workspace tests: `cargo fmt --check`, the license copies, `clippy` and `rustdoc` with warnings denied, and the tests. The wasm32 build, cargo-audit, Miri and the MSRV check run in CI as well (`msrv=yes` adds the last here). Run it before you push.

```bash
scripts/check.sh                # ci.yml's lint job and its workspace tests
scripts/check.sh features=yes   # ... and ssruntime's feature sets one at a time, native and wasm32
scripts/check.sh msrv=yes       # ... and `cargo check` with the rust-version toolchain
scripts/check.sh docs=yes       # ... and the Player Porting Guide in both locales
```

Every check runs, and the summary at the end lists each one as passed, FAILED or skipped. A check whose prerequisite is missing, such as the wasm32 target or the MSRV toolchain, is skipped rather than failed, so read the summary, not the exit status. Add `features=yes` when you change code behind a feature: `--all-features` compiles every feature at once, so it hides code that only one set compiles or leaves unused.

If you run a command by hand, keep its `--all-features`: the test modules sit behind `feature = "std"`, so without it `clippy` never lints them and a plain `cargo test` passes without running them.

### Generated code
The `.fbs` schemas under `libs/ssab/fbs/` and `libs/ssruntime/fbs/` have generated Rust bindings: after changing one, run that library's `scripts/generate.sh` (or `.ps1`). The build generates the C headers and the C# bindings with `cbindgen` and `csbindgen`, which are build-dependencies, so neither needs installing; [FFI and C-API](#ffi-and-c-api) says where they are written and when to commit them.

### Documentation site
The Player Porting Guide under `libs/ssruntime/docs` is published as a site, and [libs/ssruntime/docs/CONTRIBUTING.ja.md](./libs/ssruntime/docs/CONTRIBUTING.ja.md) (in Japanese) sets the rules for writing and translating its pages. If you edit a page there, build both locales before you finish, with `scripts/check.sh docs=yes` or the commands below. CI builds them too, both `--strict`, on a pull request that touches the site's sources, on the release PR and weekly. Set the toolchain up once with `scripts/prepare-docs.sh`, then:

```bash
scripts/build-docs.sh              # generate the locale trees, build en then ja, both --strict
scripts/build-pages.sh serve=yes   # the published tree: that build + rustdoc at /api/, on :8000
```

`zensical serve` validates nothing, `--strict` or not. The published site is this same `build-pages.sh`, which `pages.yml` runs when a release is published.

## Coding Standards

### Rust Guidelines
- Use `rustfmt` to format your code: `cargo fmt`.
- Fix what `clippy` reports; `scripts/check.sh` runs it the way CI does.
- Follow [Idiomatic Rust](https://github.com/mre/idiomatic-rust) practices.

### FFI and C-API
Since this SDK provides C-compatible interfaces (`ssruntime`, `ssconverter`), please pay attention to:
- **Binary Compatibility**: Avoid breaking changes to the C-API unless absolutely necessary.
- **Header Generation**: `build.rs` in `ssruntime-ffi` and `ssconverter-ffi` writes the C header and the C# bindings into the crate's `target/`, and `cargo test` compares them with the copies committed in its `bindings/`. If you change the C-API on purpose, or only a `///` comment on an `extern "C"` function or a `#[repr(C)]` type (both files carry those comments) or an `ffi` module's `//!` doc (it opens the header), run `UPDATE_BINDINGS=1 cargo test -p ssruntime-ffi --test bindings` (or `-p ssconverter-ffi`) and commit the copies, so the change shows in the pull request.
- **Memory Management**: Clearly document who owns the memory across the FFI boundary.

#### Naming the C-API

Every Player reads the C-API while writing its own public API, so a name settled here is copied into every Player repository.

- **Every exported symbol is `ss_<object>_<verb>_<noun>`.** The object is `runtime` for anything that takes a context, and `resource`, `effect_slot` or `instance_slot` for the three handle types that own themselves.
- **The verb comes first and is never omitted**: `get_` / `set_` / `clear_` / `find_` / `add_` / `load_` / `bind_` / `cancel_` / `fade_`. A reader and its writer spell the noun identically — `ss_runtime_get_frame_rate` against `ss_runtime_set_frame_rate`, never `get_fps`. A hash-keyed twin adds `_by_hash` and changes nothing else.
- **The noun is the stem from the vocabulary in the [Player API Conventions](./libs/ssruntime/docs/20_design/30_api_conventions.md).** Where the C-API and that vocabulary disagree, the C-API is wrong: a Player should only ever re-spell a name (`ss_runtime_just_looped` → `justLooped`), never translate it.
- **An out-parameter that points at runtime-owned memory is `const`.** The buffer belongs to the context until the next call that rebuilds it.
- **Failure crosses as a code, never as a string.** A call reports failure by its return value; the caller reads the reason with `ss_runtime_get_last_error` and turns it into text with `ss_runtime_error_message`. The codes in `libs/ssruntime/src/error.rs` are a compatibility surface of their own.

---
By contributing to this project, you agree that your contributions will be licensed under the project's [LICENSE.md](./LICENSE.md).

<br>
<br>

---

# SpriteStudio-SDK への貢献について

> [!IMPORTANT]
> コントリビューターライセンス契約（CLA）の準備が整うまで、プルリクエストは受け付けていません。バグ報告や要望は引き続き Issue で受け付けています。

SpriteStudio-SDK にご関心をお寄せいただき、ありがとうございます！ バグ報告、機能提案、ドキュメントの改善、コードの提供など、あらゆる形での貢献を歓迎します。

## 目次
- [行動規範](#行動規範)
- [貢献する方法](#貢献する方法)
- [開発環境のセットアップ](#開発環境のセットアップ)
- [コーディング規約](#コーディング規約)

## 行動規範
[CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) を参照してください。

## 貢献する方法

### 技術的制約と設計原則
本 SDK の高いパフォーマンスとポータビリティを維持するため、すべてのコード貢献は以下の核心的な原則に従う必要があります。

- **厳格な `no_std` 遵守:** コアランタイム (`libssruntime`, `libssab`) は `#![no_std]` を維持しなければなりません。ポータビリティを損なう `std` やプラットフォーム依存のライブラリへの依存は許可されません。
- **再生パスでのゼロ・アロケーション:** アニメーションの再生ループ（ホットパス）内での動的なメモリ割り当て（ヒープ確保）は厳禁です。必要なメモリはすべて事前確保するか、呼び出し側から提供される必要があります。
- **便利さよりもパフォーマンス:** オーバーヘッドを導入するような「使いやすいヘルパー関数」よりも、実行速度とメモリ効率を常に優先します。
- **SIMD フレンドリーな設計:** データ構造（特に `ssab`）は、SIMD (f32x4) によるベクトル化を念頭に設計される必要があります。カラム指向のレイアウトやフラットな配列を優先してください。
- **ステートレスな Brain（関心の分離）:** ランタイム（Brain）は状態の計算に徹し、レンダリングエンジン（Body）に関する知識を持ってはなりません。インターフェースはプリミティブかつ決定論的（Deterministic）に保ってください。
- **暗黙的より明示的:** 「マジック」や隠れた挙動を避けてください。コードは意図が明示的であり、予測可能でトレース可能である必要があります。

### バグの報告
バグを発見した場合は、Issue を作成し、以下の情報を含めてください：
- 簡潔で分かりやすいタイトル
- 再現手順
- 実行環境（OS、Rust バージョン、ターゲットプラットフォーム）
- 期待される動作と実際の動作

*注: バグ再現のために非公開の `.sspj` プロジェクトデータ等を提供いただける場合は、公開 Issue には添付せず、[SUPPORT.md](./SUPPORT.md) に記載のヘルプセンターをご利用ください。*

### 機能の提案
SDK を改善するアイデアがある場合は、開発を始める前に Issue を作成して内容を議論しましょう。

### プルリクエスト
1. リポジトリをフォークし、`develop` ブランチから作業用ブランチを作成してください。
2. 既存のスタイルや規約に従ってコードを記述してください。
3. 新しい機能やバグ修正には、対応するテストを追加してください。
4. `scripts/check.sh` が通ることを確認してください（[チェック](#チェック)を参照）。
5. 必要に応じてドキュメント (README やコメント) を更新してください。
6. `develop` ブランチに向けて、変更内容を明確に記載したプルリクエストを送信してください。

## 開発環境のセットアップ

### 前提条件
- [Rustup](https://rustup.rs/) (Stable チャンネル)。ワークスペースが対応する最も古い Rust は、ルートの `Cargo.toml` の `rust-version` です。
- [FlatBuffers コンパイラ](https://google.github.io/flatbuffers/) (`flatc`)。`.fbs` ファイルを変更する場合に必要です。ルートの `Cargo.toml` にある `flatbuffers` クレートと同じバージョンのリリースを使ってください。生成されたコードはそのクレートを呼び出します。

### ビルド
```bash
# ワークスペース全体のビルド
cargo build --release

# コンバーター CLI を実行して動作確認
cargo run -p ssconverter-cli -- <path_to_sspj>
```

### チェック
`scripts/check.sh`（PowerShell では `scripts/check.ps1`）は、`ci.yml` の lint ジョブとワークスペースのテストを実行します。`cargo fmt --check`、ライセンスの写し、警告をエラーにした `clippy` と `rustdoc`、テストです。wasm32 のビルド、cargo-audit、Miri、MSRV の確認も CI で実行されます（最後のものは `msrv=yes` でここでも実行します）。プッシュする前に実行してください。

```bash
scripts/check.sh                # ci.yml の lint ジョブとワークスペースのテスト
scripts/check.sh features=yes   # 加えて、ssruntime の feature の組み合わせを 1 つずつ（ネイティブと wasm32）
scripts/check.sh msrv=yes       # 加えて、rust-version のツールチェーンで `cargo check`
scripts/check.sh docs=yes       # 加えて、Player 移植ガイドを両ロケールでビルド
```

すべてのチェックを最後まで実行し、最後のまとめで、それぞれを passed / FAILED / skipped として示します。wasm32 のターゲットや MSRV のツールチェーンのように前提が欠けているチェックは、失敗ではなく skip になります。終了ステータスではなく、まとめを読んでください。feature の内側のコードを変えたら `features=yes` を付けてください。`--all-features` はすべての feature を一度にコンパイルするので、1 つの組み合わせでだけコンパイルされるコードや、使われなくなるコードが見えません。

コマンドを手で実行するときも `--all-features` は外さないでください。テストモジュールは `feature = "std"` 配下にあるため、付けないと `clippy` はそこを lint せず、素の `cargo test` はそれらを実行しないまま成功します。

### 生成コード
`libs/ssab/fbs/` と `libs/ssruntime/fbs/` の `.fbs` スキーマには生成された Rust バインディングがあります。変更したら、そのライブラリの `scripts/generate.sh`（または `.ps1`）を実行してください。C ヘッダーと C# バインディングは、ビルドが build-dependency の `cbindgen` と `csbindgen` で生成するので、どちらもインストールは不要です。書き出す場所とコミットするタイミングは [FFI と C-API](#ffi-と-c-api) にあります。

### ドキュメントサイト
`libs/ssruntime/docs` の Player 移植ガイドはサイトとして公開されます。ページの書き方と翻訳の規約は [libs/ssruntime/docs/CONTRIBUTING.ja.md](./libs/ssruntime/docs/CONTRIBUTING.ja.md) にあります。ページを編集したら、仕上げる前に、`scripts/check.sh docs=yes` か下のコマンドで両ロケールをビルドしてください。CI も、サイトのソースに触れるプルリクエスト、リリースのプルリクエスト、週次の実行で、両ロケールを `--strict` でビルドします。`scripts/prepare-docs.sh` で一度ツールチェーンを用意してから、以下を実行してください。

```bash
scripts/build-docs.sh              # ロケール別ツリーを生成し、en → ja の順に --strict でビルド
scripts/build-pages.sh serve=yes   # 公開ツリー: 上記 + /api/ の rustdoc を :8000 で配信
```

`zensical serve` は `--strict` を付けても何も検証しません。公開サイトもこの `build-pages.sh` で、リリースを公開したときに `pages.yml` が実行します。

## コーディング規約

### Rust のガイドライン
- `rustfmt` を使用してフォーマットを統一してください: `cargo fmt`
- `clippy` の指摘を直してください。`scripts/check.sh` が CI と同じ方法で実行します
- [慣用的な Rust](https://github.com/mre/idiomatic-rust) (Idiomatic Rust) のプラクティスに従ってください。

### FFI と C-API
この SDK は C 互換インターフェース（`ssruntime`、`ssconverter`）を提供しているため、以下の点に注意してください：
- **バイナリ互換性**: 避けられない場合を除き、C-API の破壊的変更は避けてください。
- **ヘッダー生成**: `ssruntime-ffi` と `ssconverter-ffi` の `build.rs` は、C ヘッダーと C# バインディングをそのクレートの `target/` に書き出し、`cargo test` がそれを `bindings/` にコミットされた写しと比べます。C-API を意図して変更した場合や、`extern "C"` 関数や `#[repr(C)]` 型の `///` コメント（このコメントは両方のファイルに写されます）、`ffi` モジュールの `//!` ドキュメント（ヘッダーの冒頭に写されます）だけを直した場合は、`UPDATE_BINDINGS=1 cargo test -p ssruntime-ffi --test bindings`（コンバーターは `-p ssconverter-ffi`）を実行して写しをコミットしてください。変更が PR にそのまま表示されます。
- **メモリ管理**: FFI 境界を越えてデータをやり取りする場合、メモリの所有権を明確にドキュメント化してください。

#### C-API の命名

どの Player も C-API を読みながら自分の公開 API を書くので、ここで決めた名前はすべての Player リポジトリに写されます。

- **公開するシンボルはすべて `ss_<object>_<verb>_<noun>` です。** object は、Context を取るものはすべて `runtime`、自分自身を所有する 3 つのハンドル型は `resource` / `effect_slot` / `instance_slot` です。
- **動詞を先頭に置き、省略しません**: `get_` / `set_` / `clear_` / `find_` / `add_` / `load_` / `bind_` / `cancel_` / `fade_`。読み出しと書き込みの対は名詞をまったく同じに綴ります（`ss_runtime_get_frame_rate` と `ss_runtime_set_frame_rate`。`get_fps` にはしない）。ハッシュで引く版は `_by_hash` を付けるだけで、ほかは変えません。
- **名詞は [Player API 規約](./libs/ssruntime/docs/20_design/30_api_conventions.ja.md) の語彙表の語幹です。** C-API と語彙表が食い違ったら、誤っているのは C-API です。Player は名前を綴り直すだけで済むべきで（`ss_runtime_just_looped` → `justLooped`）、訳し直させてはいけません。
- **Brain が所有するメモリを指す出力引数は `const` にします。** そのバッファは、作り直す次の呼び出しまで Context のものです。
- **失敗は文字列ではなくコードで渡します。** 呼び出しは戻り値で失敗を伝え、呼び出し側は `ss_runtime_get_last_error` で理由を取り、`ss_runtime_error_message` で文言にします。`libs/ssruntime/src/error.rs` のコードは、それ自体が互換性を保つべき面です。

---
本プロジェクトに貢献することで、あなたの貢献がプロジェクトの [LICENSE.md](./LICENSE.md) の下でライセンスされることに同意したものとみなされます。
