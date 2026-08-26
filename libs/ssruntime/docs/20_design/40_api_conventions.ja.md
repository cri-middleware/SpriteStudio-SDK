> **日本語** | [English](./40_api_conventions.md)

# Player API 規約

Brain の契約は 1 つですが、Player の公開 API は各ホスト言語で書かれます。何を揃え、何を各言語に任せるかを決めていないと、同じ機能が Player ごとに違う名前・違う語順で生えます。実際そうなりました。この文書はその境界を定めます。

**揃えるのは語幹・語順・集合です。** `get` 接頭辞の有無、プロパティかメソッドか、イベントをどう配達するか、見つからないときの戻り値は、各言語の作法に従ってください。`findPartIndex` が Rust で `Option<u32>` を返し、C# で `-1` を返すのは正しい姿です。`partIndexOf` という名前になっているのは正しくありません。

---

## 1. 引数順

オーバーライド系のメソッドは、**設定する値を `part` の直後に置き、修飾子を既定値つきで後ろに続けます。**

```
setPartVisibilityOverride(part, hide, cascade = false)
setPartColorOverride(part, color, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartColorOverrideCorners(part, corners, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartCellOverride(part, cellmapName, cellName, priority = HoldUntilNextAnimation)
```

`blendOp` は `color` の後です。この語順でなければ `blendOp` に既定値を置けず、呼び出し側は毎回 `Mix` を書かされることになります。`Mix` は零値であり、色のアルファがそのまま強度になるので「素のまま」を意味します。

既定引数を持たない言語（Rust）では、全引数を必須のままにして構いません。揃えるのは語順であって、既定値はそれが置ける言語での上乗せです。

**`corners` は 1 引数です。** 4 つの色を位置引数に展開しないでください。順序は `lt, rt, lb, rb`（左上・右上・左下・右下）で、これは C ABI の `rgba_ltrtlbrb` と同じ並びです。配列・タプル・構造体のうち、その言語で最も自然なものを使ってください。展開すると `blendOp` の居場所が Player ごとにずれます。

### C ABI とは意図的に違います

C ABI は `ss_runtime_set_part_color_override(context, part, blend_op, rgba, priority_mode)` で、`blend_op` が先です。Player はホスト向けの人間工学レイヤーなので、ここは一致させません。**C ABI を読んでそのまま真似ると規約から外れます。**

---

## 2. 名前解決 — by-name と by-index

**無印の形はパーツ名を取ります。index はその最適化であって、既定ではありません。**

ユーザーが手に持っているのはパーツ名です（エディタから来ます）。index は `findPartIndex` で引いて得るキャッシュにすぎません。既定の形が「先に引いてあること」を要求する API は順序が逆になっています。

index を渡す形の与え方は言語に任せます。

| 言語 | index を渡す形 |
|---|---|
| TypeScript | 同じメソッドに union `string \| number` |
| C# | オーバーロード `(string, …)` / `(int, …)` |
| Dart | 兄弟メソッド `…ByIndex` |
| Rust | 兄弟メソッド `…_by_index` |
| GDScript | 兄弟メソッド `…_by_index` |
| Python | どちらも受ける単一の形 |

`findPartIndex` が見つからないときの値も言語に任せます。Rust は `Option<u32>`、それ以外は C ABI と揃えて `-1` です。

---

## 3. 命名

**名前の形は、それが何を指しているかで決まります。** 迷ったらこの表で分類してから名前を決めてください。

| 指すもの | 形 | 例 |
|---|---|---|
| 継続中の動作 | 現在進行形 `is …ing` | `isPlaying` / `isPausing` / `isPlayingForward` |
| 完了して持続する状態 | 過去分詞 `is …ed` | `isFinished` |
| 1 tick 限りのパルス | `is …` を使わない | `justLooped` |
| 真偽の設定・トグル | 名詞句 + `Enabled` | `frameSkipEnabled` / `subFrameEnabled` |
| 量 | 名詞句 | `speedScale` / `loopCount` / `frameRate` |
| 失敗しうる検索 | `find…` | `findPartIndex` |

`isFinished` と `justFinished` は別物です。前者は「設定回数を再生し終えた」という残り続ける状態、後者はその立ち上がりで 1 tick だけ真になるパルスです。完了を一度だけ拾いたいなら後者を読んでください。

3 行目が最も間違えやすいところです。`isLooped` は「直近の update でループ境界を跨いだ」という 1 tick で消えるパルスであって、状態ではありません。`is` は持続する状態を連想させ、1 tick 遅れて読むというバグをそのまま誘います。同じ理由で、パルスを `has…` にするのも誤りです（「これまでに一度でも」と読めてしまいます）。

`isPausing` と `isPaused` については、英語としては後者のほうが自然だという反論があり得ます。それでも前者を採るのは、`isPlaying` と時制が揃うこと、そして Brain 自身が `ss_runtime_is_pausing` であることによります。対になる 2 つの述語で時制がずれているほうが害が大きいと判断しています。

### 標準語彙

同じ概念には同じ語幹を使ってください。大文字小文字と区切りは各言語の作法に従います。

| 概念 | 語幹 |
|---|---|
| 再生中か（一時停止中も真） | `isPlaying` |
| 一時停止で保持中か | `isPausing` |
| 設定回数を再生し終えたか（sticky） | `isFinished` |
| 直近の update でループを跨いだか（パルス） | `justLooped` |
| 直近の update で再生を完了したか（パルス） | `justFinished` |
| 実際の進行方向が前向きか | `isPlayingForward` |
| フレームスキップの有無 | `frameSkipEnabled` |
| サブフレーム補間の有無 | `subFrameEnabled` |
| 再生速度の倍率 | `speedScale` |
| ループ回数（`-1` = 無限） | `loopCount` |
| 再生フレームレート | `frameRate` |
| 再生区間の設定 | `setAnimationSection` |
| 再生方向 / 往復スタイル | `playbackDirection` / `playbackStyle` |
| パーツ名から index を引く | `findPartIndex` |
| アニメーション名の一覧 | `animationNames` |
| 現在セットアップされているアニメ | `currentAnimation` |
| セルマップ名の一覧 | `cellmapNames` |
| セル名の一覧 | `cellNames` |
| 先頭フレームを指定して再生 | `play(startFrame)` |
| 再生ヘッドの位置（小数） | `frameNo` |
| 再生ヘッドを動かす | `frameNo`（プロパティ形）/ `seek(frameNo)`（動詞形） |
| パーツ名の一覧（パーツ index 順） | `partNames` |

**`animationName` ではなく `currentAnimation`。** 前者は `animationNames` と 1 文字しか
違わず、しかもこの 2 つはホストが一緒に使う組です（一覧を出してピッカーを作り、どれが
選ばれているかを読み戻す）。Godot 自身がこの概念を `AnimationPlayer.current_animation` と
呼んでいます。`AnimatedSprite2D.animation` のほうは、自前のトランスポートを持たない
スプライトにおける選択枠です。

選択する操作は `setAnimation(name)` のままです。呼び出しではなくプロパティで選択を表す
ホストでは、そのプロパティが読み出し側の語幹を名乗ります。

**`frame` ではなく `frameNo`。** この値は小数です（サブフレーム補間で再生ヘッドは
authored フレームの間に来ます）。また `frame` は描画フレームや 1 tick が生む `FrameData`
を指す語として既に使われています。Godot の `AnimatedSprite2D.frame` は `SpriteFrames` への
**整数**インデックスなので、その綴りを借りると Godot の読み手に違う型を約束することになります。
Brain 自身の名前は `frame_no` です。

再生ヘッドを動かす側は §2 の「プロパティかメソッドか」の規則に従います。プロパティで書く
ホストでは同じ語幹への代入、動詞を好むホストでは `seek`。どちらも正しいので両方を挙げて
います。正しくないのは 3 つ目の語幹が生えることです。

**`partNames` は必須で、パーツ「個数」は代替になりません。** 個数では、イベントの
`partIndex` を人間が読める形に戻すこともできず、パーツ一覧の UI も作れません。個数は
リストの長さです。

再生方向は `playbackDirection` と `playbackStyle` の 2 つの列挙型で表します。真偽値 2 本（「逆再生か」「往復か」）にしないでください。1 回の呼び出しで原子的に設定できなくなります。

オーバーライドの優先度は Brain の列挙子名をそのまま使います。`OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` / `Permanent`。

### 用語

**`cellmap` は 1 語です。** `cellMap` や `CellMap` と綴らないでください。スキーマと本ドキュメントを通じて `cellmap` に統一されています。

---

## 4. イベント

Player は 6 つのイベントを公開します。**配達方式**はホストの作法に任せますが、**名前**と
**どれを持つか**は任せません。

### 語幹

| 何を伝えるか | 語幹 |
|---|---|
| **ユーザーデータ**のキーを通過した | `userData` |
| **シグナル**のキーを通過した | `signal` |
| **オーディオ**のキーを通過した | `audio` |
| 再生が head-out した | `animationStarted` |
| 設定したループを全て再生し終えた | `animationFinished` |
| 再生ヘッドがループ境界を跨いだ | `animationLooped` |

`on` 接頭辞・大文字小文字・語の区切りはホストに従います。Godot のシグナルなら
`animation_finished`、C# の event なら `OnAnimationFinished`、TypeScript の購読なら
`onAnimationFinished`、Python のコンストラクタ引数なら `on_animation_finished`。
揃えるのは語幹**全体**です。`animationFinished` を `finished` に縮めるのは綴りの違いでは
なく別の名前であり、エフェクトスロットもインスタンススロットも音声もテクスチャも持つ
オブジェクトの上で `finished` だけでは「何が」終わったのか分かりません。

後ろの 3 つは、§3 の語彙表が既に名前を与えている概念そのものです（`isFinished` /
`justLooped`）。**対になる名前は綴りを揃えてください。** `isFinished` という状態の隣に
`onComplete` や `onAnimationEnd` が並んでいると、1 つの概念を 2 つの語で呼んでいることになり、
どちらの名前からもう一方にも辿り着けません。

### 集合

上の 6 つが必須です。これより少ないと、他の Player では観測できることがそのホストでは
観測できなくなります。特にタイムラインの 3 つは、キーを打つこと自体の目的です。

いくつかの Player にあるが**必須ではない**ものが 3 つあります。

| 必須ではないもの | 持つ価値があるのはどんなときか |
|---|---|
| アニメ切替イベント | `setAnimation` を呼んだのはホスト自身なので、既に知っています。inspector のフィールドや宣言的な prop など**別の入り口**を持つ Player でだけ公開し、その理由をそばに書いてください |
| 毎フレームのフック | シーングラフを所有する Player でのみ意味があります。そこではパーツ追従がぶら下がる継ぎ目であり、そのフレームの行列が確定した後に呼ばれます |
| 生のイベント列 | デコード済みの 3 つを迂回する逃げ道。デコードが高価な場合や、ホストが index 自体を欲しい場合に価値があります。デコード済みイベントの代替にはなりません |

### 配達方式はホストのもの

Godot のシグナル、Unity の `event` と `UnityEvent`、TypeScript の「購読すると解除関数が返る」形、
Dart や React の代入コールバック、Ren'Py のホストが吸い出すキュー、Rust のポーリングするスライス。
どれもそのホストで正しい形です。統一しないでください。

ただし、そこから 2 つのことが導かれます。

**登録できるリスナーの数を明記してください。** これもホストの判断です（Flutter の widget
コールバックや React の prop は本来 1 本、Godot のシグナルや C# の event はそうではない）。
しかし呼び出し側が安全に確かめる方法がありません。2 本目を足せると思ったホストは、1 本目が
黙って発火しなくなったときに初めて気づきます。どちらを作ったのかを書いてください。

**コールバック機構を一切持たない Player は、代わりにパルスを公開します。** ホストが購読ではなく
ポーリングする場合、完了のエッジを取る先が他にありません。そこでは `justFinished` が
`justLooped` と並んで読み出しに加わります。`justFinished` がただ 1 本の Player にしか無いのは
このためです。イベントを持たないことの埋め合わせであって、持っている側の欠落ではありません。

---

## 5. tick はホストのクロック

ホストが駆動する Player は delta を受け取ります。**その単位はホストのクロックに従います** ——
エンジンが秒を渡すなら秒（`_process(delta)`、`Time.deltaTime`）、ミリ秒を渡すならミリ秒
（`requestAnimationFrame`）、`Duration` を持つホストなら `Duration`。入口で変換すると、
1000 倍が隠れる場所が 1 つ増えるだけです。

求めるのは**シグネチャを見て単位が分かること**です。`deltaMs` / `dt_ms` / `delta_secs` の
ような引数名か、単位を型が持っていること。裸の `delta` が通るのは、ホスト自身のクロックが
曖昧さを消している場合だけです（Godot の `delta` と Unity の `deltaTime` は、そのエンジンの
読み手にとって秒以外ではありえません）。

動詞もホストのものです。`update` と `advance` はどちらも使われていますが、quickstart 全部を
書き換えてまで揃える価値はありません。

---

## 6. 既定値

同じ機能の既定値が Player ごとに違うと、移植したときに無警告で挙動が変わります。以下は Player 側で決めてよい値ではありません。

| 対象 | 既定 | 理由 |
|---|---|---|
| オーバーライドの `priority` | `HoldUntilNextAnimation` | C ABI の既定 |
| 表示オーバーライドの `cascade` | `false` | SS7 ではキーフレームの hide が子孫に及ばないため、カスケードは opt-in |
| 色オーバーライドの `blendOp` | `Mix` | 零値。色のアルファが強度になるので「素のまま」を意味する |
| パーツ追従の非表示時挙動 | 追従を継続する | パーツが 1 フレーム隠れただけで追従対象を消さない |
| 未再生時の `isPlayingForward` | `true` | 前進が静止状態。偽を返すとホストの音声が初回フレームで塞がれる |

`subFrameEnabled` と `frameSkipEnabled` の既定は、ホストエンジンの慣習に合わせて構いません。ここだけは Player の裁量です。

### パーツ追従のオプション

パーツ追従（アタッチメント / トラッカー / アンカー）を持つ Player は、同じオプションを同じ既定で提供してください。

| オプション | 既定 | 理由 |
|---|---|---|
| position | `true` | 位置を駆動しない追従は追従ではありません |
| rotation | `true` | 手に持たせた武器は手と一緒に回ります |
| scale | **`false`** | 追従対象はパーツの**位置に置く**もので、パーツに**大きさを決めさせる**ものではありません。頭上の HP バーやネームプレートが、パーツの拡縮アニメで伸び縮みするのは大抵望まれません |
| 非表示時 | 追従を継続 | パーツが 1 フレーム隠れただけで追従対象を消さない |

**接頭辞はホストの作法に任せます。** 揃えるのは 3 つという組と既定値であって、綴りではありません。裸の `scale` がホストのトランスフォーム自身の拡縮と読み違えられる言語では、`drive`／`update` などの接頭辞を付けて構いません（Godot が `update_position` などを使うのは、`RemoteTransform2D` を踏襲しているためです）。

---

## 7. この規約から外れるとき

外れてよい場合はあります。ホスト言語やエンジンの慣習と正面から衝突するときです。その場合は**コードのそばに理由を書いてください。** 「なぜ違うのか」が書かれていない逸脱は、次に読む人には規約違反と区別がつきません。

Unity が座標系に Y-up を渡し、他の Player が Y-down を渡しているのは、正当な逸脱の例です。エンジンの座標系がそうなっているからで、そのことがコードに書かれています。
