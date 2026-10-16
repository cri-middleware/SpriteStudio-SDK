<!--
Thanks for contributing! See CONTRIBUTING.md for the constraints this project works under
(`no_std` core, no allocation in the playback hot path, a stable C ABI).
CONTRIBUTING.md に本プロジェクトの制約が記載されています。日本語でも英語でも構いません。
-->

## What does this change? / 変更内容

<!-- What it does and why. Link any related issue: Fixes #123 -->

## How was it verified? / 検証方法

<!--
Which checks ran, and on what. If you could not run them, say so —
that is useful information, not a problem.
実行した確認と環境。実行できなかった場合はその旨をお書きください。
-->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features` — without `--all-features` the `std`-gated tests do not run, and it still passes
- [ ] Converted a real project: `cargo run -p ssconverter-cli -- <project.sspj>`
- [ ] `scripts/build-docs.sh`, if the porting guide changed
- [ ] Not run — see notes below / 未実行（理由は下記）

**Environment / 環境:** OS, Rust version, target

## Checklist

- [ ] Scoped to one thing. Unrelated fixes are easier to review as their own PR.
- [ ] Follows the style of the files it touches.
- [ ] Docs updated if behaviour changed.
- [ ] A changed porting-guide page follows the guide's [authoring rules](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssruntime/docs/CONTRIBUTING.ja.md) (in Japanese).
- [ ] Anything that exists twice changed twice — a page and its translation, a `.sh` and its `.ps1`.
- [ ] `ssruntime` and `ssab` stay `#![no_std]` (`core` and `alloc` only), and the playback hot path allocates nothing.
- [ ] C ABI: no breaking change unless unavoidable, headers regenerated, names follow `ss_<object>_<verb>_<noun>`.
- [ ] A changed `.fbs` has its generated code regenerated with the library's `scripts/generate.sh` and committed.

## Anything reviewers should know? / レビュー時の注意点

<!-- Trade-offs you made, alternatives you rejected, parts you are unsure about. -->
