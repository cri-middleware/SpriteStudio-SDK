# Releasing SpriteStudio-SDK

Maintainers only. Contributors do not run any of this — see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Building the documentation site
The Player Porting Guide under `libs/ssruntime/docs` is published as a site. Set the toolchain up once with `scripts/prepare-docs.sh`, then:

```bash
scripts/build-docs.sh              # generate the locale trees, build en then ja, both --strict
scripts/build-pages.sh serve=yes   # the published tree: that build + rustdoc at /api/, on :8000
```

`pages.yml` runs `build-pages.sh` with every option spelled out, and nothing builds the docs on a pull request, so a local run is the only gate. `zensical serve` validates nothing, `--strict` or not.

## Cutting a release

A release is a tag, pushed first and built second. Bump `[workspace.package] version` in the root `Cargo.toml` to match the tag and date the `CHANGELOG.md` heading in the same commit — that version is the string every Player pins in its own `scripts/SDK_VERSION.txt`.

```bash
git tag v7.0.0-beta.1 && git push origin v7.0.0-beta.1
# Actions -> release -> "Use workflow from" = the tag, upload_release = true -> a draft Release
```

Nothing passes the tag around: `build-release.sh` reads `GITHUB_REF_NAME`, or the tag `HEAD` is checked out at, and stamps it into every archive's `VERSION`. `upload_release=true` from anything but a `v*` tag fails the run rather than skipping the Release quietly.

`scripts/build-release.sh` is the packaging phase on its own and builds nothing, so any past run's artifacts replay through it in seconds:

```bash
gh run download <run-id> -D artifacts
scripts/build-release.sh
```

`verify=yes`, the default, re-opens the finished zips and checks what has broken releases before: an empty archive, a tree that lost the root layout the Players' `download-sdk.sh` navigate, a macOS `libssruntime.framework` whose version symlinks the zip dereferenced, a `SHA256SUMS` covering the wrong set.

<br>

---

# SpriteStudio-SDK のリリース手順

維持者向けです。コントリビューターがこの手順を実行することはありません（[CONTRIBUTING.md](./CONTRIBUTING.md) を参照してください）。

## ドキュメントサイトのビルド
`libs/ssruntime/docs` の Player 移植ガイドはサイトとして公開されます。`scripts/prepare-docs.sh` で一度ツールチェーンを用意してから、以下を実行してください。

```bash
scripts/build-docs.sh              # ロケール別ツリーを生成し、en → ja の順に --strict でビルド
scripts/build-pages.sh serve=yes   # 公開ツリー: 上記 + /api/ の rustdoc を :8000 で配信
```

`pages.yml` は `build-pages.sh` をすべてのオプションを明示して実行します。プルリクエストでは docs はビルドされないため、手元のビルドが唯一のゲートです。`zensical serve` は `--strict` を付けても何も検証しません。

## リリースの手順

リリースはタグです。先にタグを push し、ビルドはその後です。ルートの `Cargo.toml` の `[workspace.package] version` をタグに合わせ、同じコミットで `CHANGELOG.md` の見出しに日付を入れてください。このバージョン文字列は、各 Player が `scripts/SDK_VERSION.txt` にピンしているものです。

```bash
git tag v7.0.0-beta.1 && git push origin v7.0.0-beta.1
# Actions -> release -> "Use workflow from" にタグを指定し、upload_release = true -> 下書き Release
```

タグは受け渡されません。`build-release.sh` が `GITHUB_REF_NAME`（ローカルでは `HEAD` のタグ）を読み、各アーカイブの `VERSION` に刻みます。`v*` タグ以外からの `upload_release=true` は、Release を黙って飛ばさずに実行を失敗させます。

`scripts/build-release.sh` はパッケージング専用で、ビルドは一切行いません。過去の実行の成果物を数秒で流し直せます。

```bash
gh run download <run-id> -D artifacts
scripts/build-release.sh
```

既定の `verify=yes` は、出来上がった zip を開き直して、過去に実際にリリースを壊したものを確認します — 空のアーカイブ、各 Player の `download-sdk.sh` が辿るルートレイアウトの欠落、zip が実体化してしまった macOS `libssruntime.framework` のバージョン symlink、対象を取り違えた `SHA256SUMS`。
