# Releasing SpriteStudio-SDK

Maintainers only. Contributors do not run any of this — see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Cutting a release

Anyone with the Write role on this repository can cut a release: every step is a branch, a pull request, a tag, a workflow run or a Release, and the signing credentials are organization secrets, so nothing depends on whose machine runs it. You need `git`, `gh` (run `gh auth login` once) and a Rust toolchain. The commands are bash; on Windows, run them in Git Bash.

A release is a tag on `main`, pushed first and built second. The version is written in one place, `[workspace.package] version` in the root `Cargo.toml`: `scripts/release-version.sh` prints it, the tag is `v` followed by it, and the release build refuses a tag that disagrees — every Player pins that tag in its own `scripts/SDK_VERSION.txt`. Branches follow git-flow: `develop` integrates, `main` holds released commits only, and `release/X.Y` lives for one release — cut from `develop` for a new `X.Y`, or from the line's latest tag for a patch, and deleted once the release is out.

```bash
VERSION=7.0.1                                       # what is being released
BRANCH=release/$(echo "$VERSION" | cut -d. -f1,2)   # release/7.0
```

### 1. Branch

A new `X.Y` is cut from `develop`. A patch release is cut from the line's latest tag, whether or not an earlier `release/X.Y` is still there, and its fixes are committed or cherry-picked onto it.

```bash
git fetch origin
git switch -c "$BRANCH" origin/develop           # a new X.Y
git switch -C "$BRANCH" v7.0.0                   # a patch release: the line's latest tag
```

### 2. Version and changelog

In one commit. Replace the previous version wherever it is written as a quoted string — the `[workspace.package]` line, the four `[workspace.dependencies]` requirements, the crate READMEs — and let cargo rewrite `Cargo.lock`, which CI builds `--locked`. Then date the version's heading in `CHANGELOG.md` as `## [X.Y.Z] - YYYY-MM-DD`, adding the heading on a patch release.

```bash
export OLD=$(scripts/release-version.sh) VERSION
git grep -lF "\"$OLD\"" -- ':!CHANGELOG.md' ':!Cargo.lock' | xargs perl -pi -e 's/"\Q$ENV{OLD}\E"/"$ENV{VERSION}"/g'
cargo update --workspace
# date the heading in CHANGELOG.md, then:
git diff
git commit -am "chore(release): v$VERSION"
```

### 3. Pull request and QA build

Open the pull request into `main`, then dispatch the release workflow from the branch. With `upload_release` left false it builds, signs and assembles all seven platforms exactly as the release will, keeps the zips on the run (`spritestudio-sdk-release-dist`), and creates no Release. Fix anything on the branch and repeat until both this run and the pull request's CI, which is the repository's full check suite, are green.

```bash
git push -u origin "$BRANCH"
gh pr create -B main --title "Release v$VERSION" --body ""
gh workflow run release.yml --ref "$BRANCH"
gh run watch                                      # the run takes a few seconds to appear
```

A missing signing secret does not fail the macOS and iOS jobs: they skip signing or notarization, say so in the log, and pass. In this run and in step 5, this must print nothing:

```bash
gh run view <run-id> --log | grep -E 'not set|Skipping' | grep -v 'echo '   # <run-id>: the one gh run watch shows
```

### 4. Merge and tag

Merge with a merge commit, never a squash or a rebase, so that `main` keeps the commits the QA build built rather than copies of them. Tag `main` only when its tree is the one the QA build built. It differs only when `main` held commits the branch did not; merge `main` into the branch and go back to step 3.

```bash
gh pr merge "$BRANCH" --merge
git switch main && git pull --ff-only
git diff --stat HEAD^2 HEAD                           # must print nothing
TAG=v$(scripts/release-version.sh) && echo "$TAG"
git tag -a "$TAG" -m "$TAG" && git push origin "$TAG"
```

### 5. Build the Release

Dispatch from the tag. The run creates a **draft** Release carrying every zip, `SHA256SUMS` and generated notes; nothing is public yet. Check the signing as in step 3.

```bash
gh workflow run release.yml --ref "$TAG" -f upload_release=true
gh run watch
```

Nothing passes the tag around: `build-release.sh` reads `GITHUB_REF_NAME`, or the tag `HEAD` is checked out at, and stamps it into every archive's `VERSION`. `upload_release=true` from anything but a `v*` tag fails the run rather than skipping the Release quietly.

### 6. Publish

Open the draft on the [Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) page, check that every platform's zip and `SHA256SUMS` are attached, edit the notes, choose **Set as a pre-release** or **Set as the latest release** (a tag with a `-` can be either), and **Publish release**. Publishing also starts `pages.yml`, which deploys the [documentation site](https://cri-middleware.github.io/SpriteStudio-SDK/). Watch the run through, then open the site: a failed run leaves the Release published and the site as it was. Once the cause is fixed, re-run it; a re-run builds the same tag.

```bash
gh run watch                                      # pages.yml; the run takes a few seconds to appear
```

### 7. Merge back

Merge `main` into `develop`. After a patch release this conflicts where `develop` has moved on: keep `develop`'s version and its `Unreleased` heading, with the patch's section below it. When the next version is decided, open it on `develop` the way step 2 does, under a `## [X.Y.Z] - Unreleased` heading.

```bash
git switch develop && git pull --ff-only
git merge --no-ff main && git push origin develop
git push origin --delete "$BRANCH"
```

If the button on its pull request already deleted the release branch, the last line only reports that the branch does not exist. Nothing needs it afterwards: a patch release cuts it again from the tag.

## Undoing a release

Until the Release is published, it can be undone. A run that failed for a transient reason is dispatched again from the same tag once its draft is deleted. A fault that needs new commits takes the tag down too: delete the draft, run `git push origin ":refs/tags/$TAG" && git tag -d "$TAG"`, fix it on the branch — restore it from its pull request if it was deleted — and go back to step 3.

**Once published, the tag and its assets are permanent** — six Players download them by that tag — so a fix is a new patch version.

## Replaying the packaging

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

## リリースの手順

このリポジトリの Write ロールを持つ人なら誰でもリリースできます。どの手順もブランチ・プルリクエスト・タグ・ワークフローの実行・Release のいずれかで、署名の資格情報は組織の Secret なので、誰の手元で実行しても結果は変わりません。必要なのは `git`、`gh`（最初に一度 `gh auth login`）、Rust ツールチェーンです。コマンドは bash で書いています。Windows では Git Bash で実行してください。

リリースは `main` 上のタグです。先にタグを push し、ビルドはその後です。バージョンが書かれているのはルートの `Cargo.toml` の `[workspace.package] version` の 1 か所だけで、`scripts/release-version.sh` がそれを出力し、タグはその頭に `v` を付けたものです。食い違うタグはリリースビルドが拒否します。各 Player は `scripts/SDK_VERSION.txt` でこのタグにピンしています。ブランチは git-flow です。`develop` が統合ブランチで、`main` にはリリース済みのコミットだけが入ります。`release/X.Y` は 1 回のリリースの間だけ使います。新しい `X.Y` なら `develop` から、パッチならその系統の最新のタグから切り、リリースが済んだら消します。

```bash
VERSION=7.0.1                                       # リリースするバージョン
BRANCH=release/$(echo "$VERSION" | cut -d. -f1,2)   # release/7.0
```

### 1. ブランチ

新しい `X.Y` は `develop` から切ります。パッチリリースは、以前の `release/X.Y` が残っているかどうかに関係なく、その系統の最新のタグから切り、修正をそこへコミットするか cherry-pick します。

```bash
git fetch origin
git switch -c "$BRANCH" origin/develop           # 新しい X.Y
git switch -C "$BRANCH" v7.0.0                   # パッチリリース: その系統の最新のタグ
```

### 2. バージョンと CHANGELOG

1 コミットで行います。前のバージョンが引用符付きの文字列として書かれている箇所 — `[workspace.package]` の行、`[workspace.dependencies]` の 4 つの要求、各クレートの README — をすべて置き換え、`Cargo.lock` を cargo に書き直させます（CI は `--locked` でビルドします）。続けて `CHANGELOG.md` のそのバージョンの見出しに日付を入れ、`## [X.Y.Z] - YYYY-MM-DD` とします。パッチリリースでは見出しを足します。

```bash
export OLD=$(scripts/release-version.sh) VERSION
git grep -lF "\"$OLD\"" -- ':!CHANGELOG.md' ':!Cargo.lock' | xargs perl -pi -e 's/"\Q$ENV{OLD}\E"/"$ENV{VERSION}"/g'
cargo update --workspace
# CHANGELOG.md の見出しに日付を入れてから:
git diff
git commit -am "chore(release): v$VERSION"
```

### 3. プルリクエストと QA ビルド

`main` へのプルリクエストを作り、ブランチからリリースワークフローを実行します。`upload_release` を false のままにすると、7 プラットフォームすべてをリリースと同じようにビルド・署名・組み立てし、zip を run に残し（`spritestudio-sdk-release-dist`）、Release は作りません。問題があればブランチで直し、この run と、リポジトリの全チェックを回すプルリクエストの CI の両方が緑になるまで繰り返します。

```bash
git push -u origin "$BRANCH"
gh pr create -B main --title "Release v$VERSION" --body ""
gh workflow run release.yml --ref "$BRANCH"
gh run watch                                      # run が一覧に出るまで数秒かかる
```

署名の Secret が欠けていても macOS と iOS のジョブは失敗しません。署名や公証を飛ばし、その旨をログに出して成功します。この run と手順 5 の run で、次のコマンドが何も出力しないことを確認してください。

```bash
gh run view <run-id> --log | grep -E 'not set|Skipping' | grep -v 'echo '   # <run-id> は gh run watch が表示するもの
```

### 4. マージとタグ

マージはマージコミットで行い、squash や rebase は使いません。QA ビルドしたコミットの写しではなく、そのものを `main` に残すためです。タグは、`main` のツリーが QA ビルドしたものと同じときにだけ打ちます。違うのは `main` にブランチが持たないコミットがあった場合だけで、そのときは `main` をブランチへマージして手順 3 に戻ります。

```bash
gh pr merge "$BRANCH" --merge
git switch main && git pull --ff-only
git diff --stat HEAD^2 HEAD                           # 何も出力されないこと
TAG=v$(scripts/release-version.sh) && echo "$TAG"
git tag -a "$TAG" -m "$TAG" && git push origin "$TAG"
```

### 5. Release のビルド

タグからワークフローを実行します。すべての zip と `SHA256SUMS`、自動生成のリリースノートを持つ**下書き**の Release が作られます。まだ何も公開されていません。署名は手順 3 と同じように確認します。

```bash
gh workflow run release.yml --ref "$TAG" -f upload_release=true
gh run watch
```

タグは受け渡されません。`build-release.sh` が `GITHUB_REF_NAME`（ローカルでは `HEAD` のタグ）を読み、各アーカイブの `VERSION` に刻みます。`v*` タグ以外からの `upload_release=true` は、Release を黙って飛ばさずに実行を失敗させます。

### 6. 公開

[Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) ページで下書きを開き、全プラットフォームの zip と `SHA256SUMS` が添付されていることを確かめ、リリースノートを編集し、**Set as a pre-release** か **Set as the latest release** を選んで（`-` を含むタグはどちらも選べます）、**Publish release** を押します。公開すると `pages.yml` が動き、[ドキュメントサイト](https://cri-middleware.github.io/SpriteStudio-SDK/) をデプロイします。run を最後まで見届けてから、サイトを開いて確かめます。run が失敗しても Release は公開されたままで、サイトは元のまま残ります。原因を直したら run を再実行します。再実行は同じタグをビルドします。

```bash
gh run watch                                      # pages.yml。run が一覧に出るまで数秒かかる
```

### 7. develop へ戻す

`main` を `develop` へマージします。パッチリリースの後は、`develop` が先へ進んだ箇所で衝突します。`develop` のバージョンと `Unreleased` の見出しを残し、パッチの節をその下に置いてください。次のバージョンが決まったら、手順 2 と同じやり方で `develop` に `## [X.Y.Z] - Unreleased` の見出しを立てます。

```bash
git switch develop && git pull --ff-only
git merge --no-ff main && git push origin develop
git push origin --delete "$BRANCH"
```

プルリクエストのページのボタンでリリースブランチを消してあれば、最後の行は「ブランチが無い」と出るだけです。パッチリリースはタグから切り直すので、この後ブランチが要ることはありません。

## リリースの取り消し

Release を公開するまでは取り消せます。一時的な理由で失敗した run は、下書きを消してから同じタグでもう一度実行します。新しいコミットが要る不具合ならタグも取り下げます。下書きを消し、`git push origin ":refs/tags/$TAG" && git tag -d "$TAG"` を実行し、ブランチで直して（消していたらプルリクエストのページから復元して）手順 3 に戻ります。

**公開した後のタグとアセットは恒久的です。** 6 つの Player がそのタグで取得するので、修正は新しいパッチバージョンで出します。

## パッケージングの再実行

`scripts/build-release.sh` はパッケージング専用で、ビルドは一切行いません。過去の実行の成果物を数秒で流し直せます。

```bash
gh run download <run-id> -D artifacts
scripts/build-release.sh
```

既定の `verify=yes` は、出来上がった zip を開き直して、過去に実際にリリースを壊したものを確認します — 空のアーカイブ、各 Player の `download-sdk.sh` が辿るルートレイアウトの欠落、zip が実体化してしまった macOS `libssruntime.framework` のバージョン symlink、対象を取り違えた `SHA256SUMS`。
