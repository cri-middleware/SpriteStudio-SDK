# Releasing SpriteStudio-SDK

Maintainers only. Contributors do not run any of this — see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Cutting a release

Anyone with the Write role on this repository can cut a release: every step is a branch, a pull request, a tag, a workflow run or a Release, and the signing credentials are organization secrets, so nothing depends on whose machine runs it. You need `git`, `gh` 2.87 or later (run `gh auth login` once), a Rust toolchain and Python 3. The repository needs GitHub Pages enabled with GitHub Actions as its source, and a `v*` tag rule on its `github-pages` environment, or publishing a Release deploys nothing; both are settings a repository admin makes. The commands are bash: in zsh, run `setopt interactive_comments` first, so that the `#` comments are not taken as arguments; on Windows, use Git Bash. Steps 1, 4 and 7 switch branches, so run them in a checkout of their own — a clone, or one made with `git worktree add` — not in one where other work is going on.

A release is a tag on `main`, pushed first and built second. The version is written in one place, `[workspace.package] version` in the root `Cargo.toml`: `scripts/release-version.sh` prints it, the tag is `v` followed by it, and the release build refuses a tag that disagrees — the six Players that download the SDK pin that tag. Branches follow git-flow: `develop` integrates, `main` holds released commits only, and `release/X.Y` lives for one release — cut from `develop` when the release ships `develop`'s work, or from the line's latest tag when it ships only fixes on top of it, and deleted once the release is out.

Every block runs as written in a new shell: it works out what it needs from the checkout, git and GitHub. The one value set by hand is the version being released, written `<version>` in steps 1 and 2 — fill it in, as in `7.0.1`; left as it is, the line stops with a syntax error. A value that would not stop a command by itself when empty is read as `${VAR:?}`: an empty `OLD` in step 2, for one, would match every `""` in the repository.

An agent can run every block, but a person sorts the scan's matches in step 3, and two steps are a person's to decide: the merge in step 4, which puts the release on `main`, and step 6, which publishes it for good.

### 1. Branch

A release that ships `develop`'s work is cut from `develop`: a new `X.Y`, or `7.0.0` after `7.0.0-beta.1`. A patch release ships only fixes: it is cut from the line's latest tag, and its fixes are committed or cherry-picked onto it. Either way, an earlier `release/X.Y` that is still there is replaced. Run the block that fits.

For `develop`'s work:

```bash
VERSION=<version>
BRANCH=release/$(echo "${VERSION:?}" | cut -d. -f1,2)
git fetch origin
git switch -C "$BRANCH" origin/develop
```

For a patch:

```bash
VERSION=<version>
BRANCH=release/$(echo "${VERSION:?}" | cut -d. -f1,2)
git fetch origin --tags
BASE=$(git -c versionsort.suffix=- tag --list "v${BRANCH#release/}.*" --sort=-v:refname | head -n 1)   # the line's latest tag
git switch -C "$BRANCH" "${BASE:?}"
```

### 2. Version and changelog

In one commit. Replace the previous version wherever it is written as a quoted string — the `[workspace.package]` line, the four `[workspace.dependencies]` requirements, the crate READMEs — and let cargo rewrite `Cargo.lock`, which CI builds `--locked`. `tests/` is left out: its SpriteStudio projects write the editor's version as a quoted string too, as in `editorVersion="7.1.0"`. `<version>` is the one from step 1.

```bash
VERSION=<version>
export OLD=$(scripts/release-version.sh) VERSION="${VERSION:?}"
git grep -lF "\"${OLD:?}\"" -- ':!CHANGELOG.md' ':!Cargo.lock' ':!tests' | xargs perl -pi -e 's/"\Q$ENV{OLD}\E"/"$ENV{VERSION}"/g'
cargo update --workspace
```

Then date the version's heading in `CHANGELOG.md` as `## [X.Y.Z] - YYYY-MM-DD`, adding the heading if the version has none yet, look over the change, and commit it:

```bash
git diff
git commit -am "chore(release): v$(scripts/release-version.sh)"
```

### 3. Pull request and QA build

Open the pull request into `main`:

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git push -u origin "${BRANCH:?}"
gh pr create -B main -H "${BRANCH:?}" --title "Release v$(scripts/release-version.sh)" --body ""
```

Then dispatch the release workflow from the branch. A run from a branch builds, signs and assembles every platform exactly as the release will, keeps the zips on the run (`spritestudio-sdk-release-dist`), and creates no Release. Fix anything on the branch and run this block again, until the run, the pull request's CI and the documentation build below all pass.

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git push origin "${BRANCH:?}" &&
  RUN=$(gh workflow run release.yml --ref "${BRANCH:?}" | sed 's|.*/||')
gh run watch "${RUN:?}" --exit-status
```

A run from a `release/` branch or a `v*` tag needs the signing secrets: without one, the macOS or iOS job fails at its first step and names what is missing. No change on the branch fixes that: the organization's signing secrets are not reaching this repository. An organization owner decides which repositories they reach, and on GitHub Free they cannot reach a private one. Dispatch again once they do. A run from any other branch skips signing or notarization instead, says so in the log, and passes.

Nothing builds the documentation site on a pull request, and `pages.yml` builds it from the tag after the Release is published, where a fault in a page cannot be fixed by re-running it. Build it here as `pages.yml` will:

```bash
scripts/build-pages.sh                            # both locales --strict, then rustdoc at /api/
```

Last, scan the branch for code copied from others. `scripts/scan-snippets.sh` matches the files tracked at `HEAD` against public open-source code with SCANOSS, sending their fingerprints rather than the code; it writes its report to `qa/scanoss-<commit>.json` and prints the matches. It runs on this machine, and needs Python 3: SCANOSS's free service limits calls per address, and GitHub's runners share addresses it has often limited already. `provenance.yml` runs ScanCode, which finds copyright and license notices, and keeps its report and a summary on the run (`provenance`); the block downloads them into `qa/<run ID>` and prints the summary. **A person sorts the matches.** A match with the earlier SpriteStudio Players and SDK is our own code. Anything else is rewritten, unless its license, read at its source, is compatible with BSD-3-Clause and the code keeps the notice that license asks for. After any change on the branch, run the block again.

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git push origin "${BRANCH:?}" &&
  RUN=$(gh workflow run provenance.yml --ref "${BRANCH:?}" | sed 's|.*/||') &&
  scripts/scan-snippets.sh &&
  gh run watch "${RUN:?}" --exit-status &&
  gh run download "${RUN:?}" -n provenance -D "qa/$RUN" &&
  cat "qa/$RUN/summary.md"
```

### 4. Merge and tag

Merge with a merge commit, never a squash or a rebase, so that `main` keeps the commits the QA build built rather than copies of them. Before merging, check that a QA run built the branch's head green, and that merging it changes nothing in its tree. If it would, `main` holds changes the branch does not, and the tag would go on a tree no QA run built: merge `main` into the branch and go back to step 3.

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git fetch origin
SHA=$(git rev-parse --verify -q "origin/${BRANCH:?}")
gh run list -w release.yml -c "${SHA:?}" -s success -L 1                                             # must list a run
git diff --stat "origin/${BRANCH:?}" "$(git merge-tree --write-tree origin/main "origin/$BRANCH")"   # must print nothing
```

**A person runs the merge**, which puts the release on `main`: choose **Create a merge commit** on the pull request's page, or run

```bash
BRANCH=$(git branch --show-current | grep '^release/')
gh pr merge "${BRANCH:?}" --merge
```

Then tag `main`. The tag goes on the merge commit only when its tree is the branch's, which `git diff` confirms by printing nothing:

```bash
git switch main && git pull --ff-only &&
  git diff --stat --exit-code HEAD^2 HEAD &&
  TAG=v$(scripts/release-version.sh) &&
  git tag -a "$TAG" -m "$TAG" && git push origin "$TAG"
```

### 5. Build the Release

Dispatch from the tag on `main`. The run creates a **draft** Release carrying every zip, `SHA256SUMS` and generated notes; nothing is public yet. As in step 3, a missing signing secret fails the run.

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
RUN=$(gh workflow run release.yml --ref "${TAG:?}" -f upload_release=true | sed 's|.*/||')
gh run watch "${RUN:?}" --exit-status
```

Nothing passes the tag around: `build-release.sh` reads `GITHUB_REF_NAME`, or the tag `HEAD` is checked out at, and stamps it into every archive's `VERSION`. A tag that already has a Release, draft or published, keeps it as it is: the run builds, says so in a notice and creates nothing, so delete a draft before dispatching again ([Undoing a release](#undoing-a-release)). With `-f upload_release=false` a run from the tag only builds, its files kept on the run.

### 6. Publish

**A person runs this step**: the notes are a judgment, and a published Release is permanent. Open the draft on the [Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) page, check that `SHA256SUMS` and every file it lists are attached — the `spritestudio-sdk-*-libs.zip` and the `ssconverter-cli-*.zip` — edit the notes, and **Publish release** as the draft is marked: `scripts/release-channel.sh` made it a pre-release when a full release is already tagged, and left it to become Latest otherwise. Only a Release that becomes Latest deploys the site.

Publishing also starts `pages.yml`, which deploys the [documentation site](https://cri-middleware.github.io/SpriteStudio-SDK/). Watch the run through, then open the site: a failed run leaves the Release published and the site as it was. Once the cause is fixed, re-run it with `gh run rerun "${RUN:?}"`; a re-run builds the same tag.

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
RUN=$(gh run list -w pages.yml -b "${TAG:?}" -L 1 --json databaseId -q '.[0].databaseId')
gh run watch "${RUN:?}" --exit-status             # RUN is empty until the run appears: repeat the last two lines
```

### 7. Merge back

Merge `main` into `develop`. After a patch release this conflicts where `develop` has moved on: keep `develop`'s version and its `Unreleased` heading, with the patch's section below it. When the next version is decided, open it on `develop` the way step 2 does, under a `## [X.Y.Z] - Unreleased` heading.

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
git switch develop && git pull --ff-only &&
  git merge --no-ff origin/main && git push origin develop
git push origin --delete "release/$(echo "${TAG:?}" | sed 's/^v//' | cut -d. -f1,2)"
```

If the button on its pull request already deleted the release branch, the last line only reports that the branch does not exist. Nothing needs it afterwards: the line's next release cuts it again in step 1.

## Undoing a release

Until the Release is published, it can be undone. The draft goes first, if the run made one:

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
gh release delete "${TAG:?}" --yes
```

A run that failed for a transient reason is then dispatched again from the same tag, with step 5's block. A fault that needs new commits takes the tag down too, on GitHub and here; then fix it on the branch — restore it from its pull request if it was deleted — and go back to step 3.

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
git push origin ":refs/tags/${TAG:?}" && git tag -d "${TAG:?}"
```

**Once published, the tag and its assets are permanent** — six Players download them by that tag — so a fix is a new patch version.

## Replaying the packaging

`scripts/build-release.sh` is the packaging phase on its own and builds nothing, so a run's artifacts replay through it in seconds — any run from the last week, as the workflow keeps them for seven days. `<run-id>` is that run's ID. The block checks out the commit the run built first, because the script reads the tag it stamps into `VERSION` from the checkout.

```bash
RUN=<run-id>
git checkout "$(gh run view "${RUN:?}" --json headSha -q .headSha)" &&
  gh run download "${RUN:?}" -D artifacts &&
  scripts/build-release.sh
```

`verify=yes`, the default, re-opens the finished zips and checks what has broken releases before: an empty archive, a tree that lost the root layout the Players' `download-sdk.sh` navigate, a macOS `libssruntime.framework` whose version symlinks the zip dereferenced, a `SHA256SUMS` covering the wrong set.

<br>

---

# SpriteStudio-SDK のリリース手順

維持者向けです。コントリビューターがこの手順を実行することはありません（[CONTRIBUTING.md](./CONTRIBUTING.md) を参照してください）。

## リリースの手順

このリポジトリの Write ロールを持つ人なら誰でもリリースできます。どの手順もブランチ・プルリクエスト・タグ・ワークフローの実行・Release のいずれかで、署名の資格情報は組織の Secret なので、誰の手元で実行しても結果は変わりません。必要なのは `git`、`gh` 2.87 以降（最初に一度 `gh auth login`）、Rust ツールチェーン、Python 3 です。リポジトリ側では、GitHub Pages を有効にしてソースを GitHub Actions にし、`github-pages` environment に `v*` のタグのルールを足しておく必要があります。どちらかが欠けると、Release を公開しても何もデプロイされません。どちらもリポジトリの admin が行う設定です。コマンドは bash です。zsh では、`#` のコメントが引数として渡されないよう、先に `setopt interactive_comments` を実行します。Windows では Git Bash を使います。手順 1・4・7 はブランチを切り替えるので、ほかの作業をしていない専用のチェックアウト（クローンか、`git worktree add` で作ったもの）で実行してください。

リリースは `main` 上のタグです。先にタグを push し、ビルドはその後です。バージョンが書かれているのはルートの `Cargo.toml` の `[workspace.package] version` の 1 か所だけで、`scripts/release-version.sh` がそれを出力し、タグはその頭に `v` を付けたものです。食い違うタグはリリースビルドが拒否します。SDK をダウンロードする 6 つの Player がこのタグにピンしています。ブランチは git-flow です。`develop` が統合ブランチで、`main` にはリリース済みのコミットだけが入ります。`release/X.Y` は 1 回のリリースの間だけ使います。`develop` の作業を出すリリースなら `develop` から、タグの上に修正だけを載せるリリースならその系統の最新のタグから切り、リリースが済んだら消します。

どのブロックも、新しいシェルで書かれたとおりに実行できます。必要なものはチェックアウト・git・GitHub から求めます。手で設定する値はリリースするバージョンだけで、手順 1 と 2 に `<version>` と書いてあります。`7.0.1` のように埋めてください。埋めずに実行すると、その行は構文エラーで止まります。空でもそれだけでは止まらない値は `${VAR:?}` で読みます。たとえば手順 2 の `OLD` が空だと、リポジトリ中の `""` すべてに一致してしまいます。

どのブロックもエージェントに実行させられますが、手順 3 の照合の一致は人が仕分け、2 つの手順は人が判断します。リリースを `main` に入れる手順 4 のマージと、取り消せない公開を行う手順 6 です。

### 1. ブランチ

`develop` の作業を出すリリースは `develop` から切ります。新しい `X.Y` や、`7.0.0-beta.1` の後の `7.0.0` がこれに当たります。パッチリリースは修正だけを出すもので、その系統の最新のタグから切り、修正をそこへコミットするか cherry-pick します。どちらの場合も、以前の `release/X.Y` が残っていれば置き換えます。当てはまる方のブロックを実行します。

`develop` の作業を出すとき:

```bash
VERSION=<version>
BRANCH=release/$(echo "${VERSION:?}" | cut -d. -f1,2)
git fetch origin
git switch -C "$BRANCH" origin/develop
```

パッチのとき:

```bash
VERSION=<version>
BRANCH=release/$(echo "${VERSION:?}" | cut -d. -f1,2)
git fetch origin --tags
BASE=$(git -c versionsort.suffix=- tag --list "v${BRANCH#release/}.*" --sort=-v:refname | head -n 1)   # その系統の最新のタグ
git switch -C "$BRANCH" "${BASE:?}"
```

### 2. バージョンと CHANGELOG

1 コミットで行います。前のバージョンが引用符付きの文字列として書かれている箇所 — `[workspace.package]` の行、`[workspace.dependencies]` の 4 つの要求、各クレートの README — をすべて置き換え、`Cargo.lock` を cargo に書き直させます（CI は `--locked` でビルドします）。`tests/` は対象から外します。そこにある SpriteStudio のプロジェクトも、`editorVersion="7.1.0"` のようにエディタのバージョンを引用符付きの文字列で書いているためです。`<version>` は手順 1 と同じものです。

```bash
VERSION=<version>
export OLD=$(scripts/release-version.sh) VERSION="${VERSION:?}"
git grep -lF "\"${OLD:?}\"" -- ':!CHANGELOG.md' ':!Cargo.lock' ':!tests' | xargs perl -pi -e 's/"\Q$ENV{OLD}\E"/"$ENV{VERSION}"/g'
cargo update --workspace
```

続けて `CHANGELOG.md` のそのバージョンの見出しに日付を入れ、`## [X.Y.Z] - YYYY-MM-DD` とします。そのバージョンの見出しがまだ無ければ足します。変更を見直してからコミットします。

```bash
git diff
git commit -am "chore(release): v$(scripts/release-version.sh)"
```

### 3. プルリクエストと QA ビルド

`main` へのプルリクエストを作ります。

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git push -u origin "${BRANCH:?}"
gh pr create -B main -H "${BRANCH:?}" --title "Release v$(scripts/release-version.sh)" --body ""
```

続けて、ブランチからリリースワークフローを実行します。ブランチからの実行では、すべてのプラットフォームをリリースと同じようにビルド・署名・組み立てし、zip を run に残し（`spritestudio-sdk-release-dist`）、Release は作りません。問題があればブランチで直し、このブロックをもう一度実行します。run、プルリクエストの CI、下のドキュメントのビルドがすべて通るまで繰り返します。

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git push origin "${BRANCH:?}" &&
  RUN=$(gh workflow run release.yml --ref "${BRANCH:?}" | sed 's|.*/||')
gh run watch "${RUN:?}" --exit-status
```

`release/` ブランチや `v*` タグからの実行には署名の Secret が必要です。欠けていると、macOS か iOS のジョブが最初のステップで失敗し、足りない Secret の名前を出します。ブランチをどう変えてもこれは直りません。組織の署名用 Secret がこのリポジトリに届いていません。どのリポジトリに届けるかは組織のオーナーが決めますが、GitHub Free では private のリポジトリには届けられません。届くようになったら実行し直してください。ほかのブランチからの実行では、署名や公証を飛ばし、その旨をログに出して成功します。

プルリクエストではドキュメントサイトを誰もビルドしません。`pages.yml` がビルドするのは Release を公開した後で、しかもタグからなので、ページの不具合は再実行では直せません。ここで `pages.yml` と同じようにビルドしておきます。

```bash
scripts/build-pages.sh                            # 両ロケールを --strict でビルドし、/api/ に rustdoc を置く
```

最後に、他者のコードを写した箇所が無いかを、ブランチで照合します。`scripts/scan-snippets.sh` は、`HEAD` で追跡しているファイルを、SCANOSS で公開されているオープンソースのコードと照合します。送るのはコードではなく指紋です。報告を `qa/scanoss-<commit>.json` に書き、一致を表示します。このスクリプトは手元で実行し、Python 3 が要ります。SCANOSS の無料のサービスはアドレスごとに回数を制限していて、GitHub のランナーはすでに制限されたアドレスを共有していることが多いためです。`provenance.yml` は ScanCode で著作権表示とライセンスの記述を見つけ、報告と一覧を run に残します（`provenance`）。ブロックはそれを `qa/<run ID>` にダウンロードし、一覧を表示します。**一致は人が仕分けます。** 以前の SpriteStudio の Player・SDK との一致は、自社のコードです。それ以外は書き直します。残せるのは、出所で確かめたライセンスが BSD-3-Clause と両立し、そのライセンスが求める表示をコードに残すときだけです。ブランチを変えたら、このブロックをもう一度実行します。

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git push origin "${BRANCH:?}" &&
  RUN=$(gh workflow run provenance.yml --ref "${BRANCH:?}" | sed 's|.*/||') &&
  scripts/scan-snippets.sh &&
  gh run watch "${RUN:?}" --exit-status &&
  gh run download "${RUN:?}" -n provenance -D "qa/$RUN" &&
  cat "qa/$RUN/summary.md"
```

### 4. マージとタグ

マージはマージコミットで行い、squash や rebase は使いません。QA ビルドしたコミットの写しではなく、そのものを `main` に残すためです。マージの前に、ブランチの先頭を QA の run が緑でビルドしていること、マージしてもブランチのツリーが変わらないことを確かめます。変わるなら `main` にブランチの持たない変更があり、どの QA の run もビルドしていないツリーにタグが付きます。そのときは `main` をブランチへマージして手順 3 に戻ります。

```bash
BRANCH=$(git branch --show-current | grep '^release/')
git fetch origin
SHA=$(git rev-parse --verify -q "origin/${BRANCH:?}")
gh run list -w release.yml -c "${SHA:?}" -s success -L 1                                             # run が 1 つ表示されること
git diff --stat "origin/${BRANCH:?}" "$(git merge-tree --write-tree origin/main "origin/$BRANCH")"   # 何も出力されないこと
```

**マージは人が行います。** リリースが `main` に入るためです。プルリクエストのページで **Create a merge commit** を選ぶか、次を実行します。

```bash
BRANCH=$(git branch --show-current | grep '^release/')
gh pr merge "${BRANCH:?}" --merge
```

続けて `main` にタグを打ちます。タグはマージコミットのツリーがブランチと同じときにだけ打たれます。`git diff` が何も出力しないことがその確認です。

```bash
git switch main && git pull --ff-only &&
  git diff --stat --exit-code HEAD^2 HEAD &&
  TAG=v$(scripts/release-version.sh) &&
  git tag -a "$TAG" -m "$TAG" && git push origin "$TAG"
```

### 5. Release のビルド

`main` 上のタグからワークフローを実行します。すべての zip と `SHA256SUMS`、自動生成のリリースノートを持つ**下書き**の Release が作られます。まだ何も公開されていません。手順 3 と同じく、署名の Secret が欠けていれば実行は失敗します。

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
RUN=$(gh workflow run release.yml --ref "${TAG:?}" -f upload_release=true | sed 's|.*/||')
gh run watch "${RUN:?}" --exit-status
```

タグは受け渡されません。`build-release.sh` が `GITHUB_REF_NAME`（ローカルでは `HEAD` のタグ）を読み、各アーカイブの `VERSION` に刻みます。Release がすでにあるタグでは、下書きでも公開済みでも、その Release に触れません。run はビルドして notice を出すだけなので、もう一度実行するときは先に下書きを消します（[リリースの取り消し](#リリースの取り消し)）。`-f upload_release=false` を付けると、タグからの実行でもビルドだけを行い、成果物を run に残します。

### 6. 公開

**この手順は人が行います。** リリースノートの中身は判断が要り、公開した Release は取り消せないためです。[Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) ページで下書きを開き、`SHA256SUMS` と、それが列挙するファイル（`spritestudio-sdk-*-libs.zip` と `ssconverter-cli-*.zip`）がすべて添付されていることを確かめ、リリースノートを編集し、下書きの印のまま **Publish release** を押します。`scripts/release-channel.sh` が、正式版のタグがすでにあれば pre-release にし、なければ Latest になるようにしてあります。サイトをデプロイするのは Latest になる Release だけです。

公開すると `pages.yml` が動き、[ドキュメントサイト](https://cri-middleware.github.io/SpriteStudio-SDK/) をデプロイします。run を最後まで見届けてから、サイトを開いて確かめます。run が失敗しても Release は公開されたままで、サイトは元のまま残ります。原因を直したら `gh run rerun "${RUN:?}"` で再実行します。再実行は同じタグをビルドします。

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
RUN=$(gh run list -w pages.yml -b "${TAG:?}" -L 1 --json databaseId -q '.[0].databaseId')
gh run watch "${RUN:?}" --exit-status             # run が出るまで RUN は空。最後の 2 行を繰り返す
```

### 7. develop へ戻す

`main` を `develop` へマージします。パッチリリースの後は、`develop` が先へ進んだ箇所で衝突します。`develop` のバージョンと `Unreleased` の見出しを残し、パッチの節をその下に置いてください。次のバージョンが決まったら、手順 2 と同じやり方で `develop` に `## [X.Y.Z] - Unreleased` の見出しを立てます。

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
git switch develop && git pull --ff-only &&
  git merge --no-ff origin/main && git push origin develop
git push origin --delete "release/$(echo "${TAG:?}" | sed 's/^v//' | cut -d. -f1,2)"
```

プルリクエストのページのボタンでリリースブランチを消してあれば、最後の行は「ブランチが無い」と出るだけです。同じ系統の次のリリースは手順 1 で切り直すので、この後ブランチが要ることはありません。

## リリースの取り消し

Release を公開するまでは取り消せます。run が下書きを作っていれば、先にそれを消します。

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
gh release delete "${TAG:?}" --yes
```

一時的な理由で失敗した run は、そのあと手順 5 のブロックで同じタグからもう一度実行します。新しいコミットが要る不具合なら、タグも GitHub と手元の両方から取り下げ、ブランチで直して（消していたらプルリクエストのページから復元して）手順 3 に戻ります。

```bash
git fetch origin --tags
TAG=$(git describe --exact-match --tags --match 'v[0-9]*' origin/main)
git push origin ":refs/tags/${TAG:?}" && git tag -d "${TAG:?}"
```

**公開した後のタグとアセットは恒久的です。** 6 つの Player がそのタグで取得するので、修正は新しいパッチバージョンで出します。

## パッケージングの再実行

`scripts/build-release.sh` はパッケージング専用で、ビルドは一切行いません。run の成果物を数秒で流し直せます。ワークフローは成果物を 7 日間保持するので、対象は直近 1 週間の run です。`<run-id>` はその run の ID です。スクリプトは `VERSION` に刻むタグをチェックアウトから読むので、ブロックは先にその run がビルドしたコミットをチェックアウトします。

```bash
RUN=<run-id>
git checkout "$(gh run view "${RUN:?}" --json headSha -q .headSha)" &&
  gh run download "${RUN:?}" -D artifacts &&
  scripts/build-release.sh
```

既定の `verify=yes` は、出来上がった zip を開き直して、過去に実際にリリースを壊したものを確認します — 空のアーカイブ、各 Player の `download-sdk.sh` が辿るルートレイアウトの欠落、zip が実体化してしまった macOS `libssruntime.framework` のバージョン symlink、対象を取り違えた `SHA256SUMS`。
