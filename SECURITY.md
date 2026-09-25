# Security Policy

## Supported Versions

SpriteStudio-SDK is pre-release: nothing is published yet, and fixes are issued against `develop`.
Once there are releases, this becomes "the latest one".

| Version | Supported |
|---|---|
| `develop` | ✅ |
| Anything else | ❌ |

## Reporting a Vulnerability

**Please do not open a public GitHub Issue for a security problem.** Report it privately through the
official inquiry form:

👉 [**CRI Middleware Inquiry Form (English)**](https://www.webtech.co.jp/contact/en.html)

The English form has no file upload. If you need to send a reproduction, use the
[Japanese Help Center form](https://www.webtech.co.jp/help/ja/spritestudio7/inquiries/ssplayer_tool/),
which takes a zip of up to 8 MB.

Please include:

- What the impact is — crash, out-of-bounds read, arbitrary code execution, information disclosure.
- Steps to reproduce, ideally with a minimal `.ssab` or `.sspj` and the smallest amount of Rust or C
  needed.
- The SDK commit or version, which part (converter, runtime, C ABI, wasm), your OS and target, and —
  if you found it through a player — which player.
- Whether the input that triggers it comes from a trusted source in your project or from somewhere
  untrusted (downloaded content, user-supplied files).

We will confirm receipt, tell you whether we can reproduce it, and let you know when a fix ships.
Please give us a chance to release that fix before disclosing publicly.

### Which repository to report to

This repository holds the converter, the runtime every SpriteStudio player shares, and their C ABI and
wasm faces. If you found the problem through a player, reporting it to that player is fine too; it will
be routed here.

## Scope Notes

**The runtime does not validate `.ssab` input.** `SsabResource::new` and `AnimationResource::new`,
the C ABI's `ss_resource_create_*` and `ss_runtime_load_ssab_*`, and the wasm `SSRuntimeResource` parse
the buffer without FlatBuffers verification; apart from the schema version, nothing is checked. A
malformed or deliberately crafted `.ssab` can therefore cause out-of-bounds reads rather than a clean
error; that is also why the two Rust constructors are `unsafe fn`, with a valid pack as their
precondition. The crate documentation and the porting guide make verifying a pack the player's job:
check the `"SSAB"` identifier and run the FlatBuffers verifier for its language before handing the
bytes over. Rust hosts can use the checked `root_as_ss_anime_binary` in the `ssab` crate.

**The converter is built for projects you author.** It reads the files a `.sspj` references, and writes
the converted packs and copies of the assets under the output directory. Image, texture and sound paths
that resolve outside the project directory are refused. The wasm converter sees only the files its host
passes in. If a tool of yours converts `.sspj` supplied by others, treat them as untrusted input.

Reports about the known gap above are welcome, but it is already known — a report that demonstrates
impact beyond it is more useful.

---

# セキュリティポリシー

## サポート対象バージョン

SpriteStudio-SDK はリリース前です。まだ何も公開されておらず、修正は `develop` に対して提供されます。リリースが始まった時点で「最新リリース」に切り替わります。

| バージョン | サポート |
|---|---|
| `develop` | ✅ |
| それ以外 | ❌ |

## 脆弱性の報告

**セキュリティに関する問題は、公開の GitHub Issue に投稿しないでください。** 再現データの添付も可能なヘルプセンターより、非公開でご報告ください。

👉 [**ヘルプセンター（お問い合わせフォーム）**](https://www.webtech.co.jp/help/ja/spritestudio7/inquiries/ssplayer_tool/)

ご報告には以下を含めてください。

- 影響の内容（クラッシュ、境界外読み取り、任意コード実行、情報漏洩など）
- 再現手順。可能であれば最小限の `.ssab` / `.sspj` と、必要最小限の Rust または C のコード
- SDK のコミットまたはバージョン、どの部分か（コンバーター、ランタイム、C ABI、wasm）、OS とターゲット、Player を通じて見つけた場合はその Player
- 引き金となる入力が、プロジェクト内の信頼できるデータか、信頼できない経路（ダウンロードコンテンツ、ユーザーが用意したファイル）由来か

受領の確認、再現可否、修正版の公開時期についてご連絡します。修正を公開するまでの間、公表をお待ちいただけますようお願いします。

### 報告先リポジトリの判断

本リポジトリは、コンバーター、すべての SpriteStudio Player が共有するランタイム、およびその C ABI と wasm を扱います。Player を通じて問題を見つけた場合は、その Player へご報告いただいても構いません。こちらへ振り分けられます。

## 既知の注意点

**ランタイムは `.ssab` の入力を検証しません。** `SsabResource::new` と `AnimationResource::new`、C ABI の `ss_resource_create_*` と `ss_runtime_load_ssab_*`、wasm の `SSRuntimeResource` は、FlatBuffers の検証を行わずにバッファを解析します。確認するのはスキーマのバージョンだけです。そのため、破損した、あるいは意図的に細工された `.ssab` は、エラーとして弾かれずに境界外読み取りを引き起こす可能性があります。Rust の 2 つのコンストラクタが `unsafe fn` なのも、正しいパックであることが呼び出しの前提になるためです。クレートのドキュメントと移植ガイドは、パックの検証を Player の責務としています。バイト列を渡す前に `"SSAB"` 識別子を確認し、その言語の FlatBuffers verifier を実行してください。Rust のホストは `ssab` クレートの検証付き `root_as_ss_anime_binary` を利用できます。

**コンバーターは、ご自身で作成したプロジェクトを前提としています。** `.sspj` が参照するファイルを読み込み、変換したパックとアセットのコピーを出力ディレクトリの下に書き出します。プロジェクトディレクトリの外を指す画像・テクスチャ・サウンドのパスは拒否します。wasm 版のコンバーターは、ホストが渡したファイルだけを参照します。他者から受け取った `.sspj` を変換するツールでは、それを信頼できない入力として扱ってください。

上記の既知の課題に関するご報告も歓迎しますが、すでに把握している内容です。これを超える影響を示すご報告のほうが有用です。
