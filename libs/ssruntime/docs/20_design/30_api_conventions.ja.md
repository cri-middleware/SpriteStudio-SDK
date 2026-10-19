> **日本語** | [English](./30_api_conventions.md)

# Player API 規約

Player の公開 API はホストの言語ごとに書かれます。このページは、すべての Player で揃えるものと、各言語の作法に任せるものの境界を決めます。

**揃えるのは、語幹・引数の順・持つべき集合です。** `get` 接頭辞の有無、プロパティかメソッドか、イベントの届け方、見つからないときの戻り値は、各言語の作法に従ってください。`findPartIndex` が Rust で `Option<u32>` を返し、C# で `-1` を返すのは正しい形です。`partIndexOf` という名前になっているのは誤りです。

---

## 1. 引数の順

オーバーライド系のメソッドは、**設定する値を `part` の直後に置き、修飾子を既定値つきで後ろに並べます。**

```
setPartVisibilityOverride(part, hide, cascade = false)
setPartColorOverride(part, color, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartColorOverrideCorners(part, corners, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartCellOverride(part, cellmapName, cellName, priority = HoldUntilNextAnimation)
```

`blendOp` は `color` の後に置きます。この順でないと `blendOp` に既定値を置けず、呼び出し側が毎回 `Mix` を書くことになります。`Mix` は零値で、色のアルファがそのまま強度になるので「素のまま」を意味します。

既定引数の無い言語（Rust）では、すべての引数を必須にしてかまいません。揃えるのは順であり、既定値は置ける言語での上乗せです。

**`corners` は 1 つの引数です。** 4 つの色を位置引数に展開しないでください。順は `lt, rt, lb, rb`（左上・右上・左下・右下）で、C-API の `rgba_ltrtlbrb` と同じです。配列・タプル・構造体のうち、その言語で最も自然なものを使ってください。展開すると、`blendOp` の位置が Player ごとにずれます。

### 1-1. C-API とは順が違う

C-API は `ss_runtime_set_part_color_override(context, part, blend_op, rgba, priority_mode)` で、`blend_op` が先です。Player はホスト向けの使いやすさを担う層なので、ここは C-API に合わせません。**C-API の順をそのまま写すと、この規約から外れます。**

---

## 2. 名前とインデックス

**無印の形はパーツ名を取ります。インデックスを取る形は最適化であって、既定ではありません。**

利用者の手元にあるのは、エディタから来たパーツ名です。インデックスは `findPartIndex` で引いて得るキャッシュにすぎません。

インデックスを渡す形の与え方は、言語に任せます。

| 言語 | インデックスを渡す形 |
|---|---|
| TypeScript | 同じメソッドに union `string \| number` |
| C# | オーバーロード `(string, …)` / `(int, …)` |
| Dart | 兄弟メソッド `…ByIndex` |
| Rust | 兄弟メソッド `…_by_index` |
| GDScript | 兄弟メソッド `…_by_index` |
| Python | どちらも受ける 1 つの形 |

`findPartIndex` が見つからないときの値も言語に任せます。Rust は `Option<u32>`、それ以外は C-API と同じ `-1` です。

---

## 3. 命名

**名前の形は、それが何を指すかで決まります。** 迷ったら、この表で分類してから名前を決めてください。

| 指すもの | 形 | 例 |
|---|---|---|
| 続いている動作 | 進行形 `is …ing` | `isPlaying` / `isPausing` / `isPlayingForward` |
| 完了して残る状態 | 過去分詞 `is …ed` | `isFinished` |
| 1 tick だけのパルス | `is …` を使わない | `justLooped` |
| 真偽の設定 | 名詞句 + `Enabled` | `frameSkipEnabled` / `subFrameEnabled` |
| 量 | 名詞句 | `speedScale` / `loopCount` / `frameRate` |
| 失敗しうる検索 | `find…` | `findPartIndex` |

`isFinished` と `justFinished` は別物です。前者は「設定回数を再生し終えた」という残り続ける状態、後者はその立ち上がりで 1 tick だけ真になるパルスです。

3 行目が最も間違えやすいところです。「直近の update でループ境界を跨いだ」は 1 tick で消えるパルスなので、`isLooped` と名付けると状態に見え、1 tick 遅れて読む誤りを招きます。同じ理由で、パルスを `has…` にするのも誤りです（「これまでに一度でも」と読めてしまいます）。

`justStarted` は、`play` / `play(startFrame)` の後に最初に走った update で 1 tick だけ真になります。`resume` では立ちません。一時停止を解くのは続きの再生で、再生の開始ではありません。

`isPaused` ではなく `isPausing` を使うのは、対になる `isPlaying` と時制を揃えるためで、Brain 自身も `ss_runtime_is_pausing` です。

### 3-1. 標準語彙

同じ概念には同じ語幹を使ってください。大文字小文字と区切りは各言語の作法に従います。

| 概念 | 語幹 |
|---|---|
| 再生中か（一時停止中も真） | `isPlaying` |
| 一時停止で保持しているか | `isPausing` |
| 設定回数を再生し終えたか（状態） | `isFinished` |
| 直近の update が `play` の後の最初の update だったか（パルス） | `justStarted` |
| 直近の update でループを跨いだか（パルス） | `justLooped` |
| 直近の update で再生を終えたか（パルス） | `justFinished` |
| 実際の進行方向が前向きか | `isPlayingForward` |
| フレームスキップの有無 | `frameSkipEnabled` |
| サブフレームの有無 | `subFrameEnabled` |
| 再生速度の倍率 | `speedScale` |
| ループ回数（`-1` = 無限） | `loopCount` |
| 再生のフレームレート | `frameRate` |
| 再生区間の設定 | `setAnimationSection` |
| ラベルによる再生区間の設定 | `setAnimationSectionByLabel` |
| 再生方向 / 往復のスタイル | `playbackDirection` / `playbackStyle` |
| パーツ名からインデックスを引く | `findPartIndex` |
| アニメーション名の一覧 | `animationNames` |
| 現在セットアップされているアニメーション | `currentAnimation` |
| セルマップ名の一覧 | `cellmapNames` |
| セル名の一覧 | `cellNames` |
| 先頭フレームを指定して再生する | `play(startFrame)` |
| 再生ヘッドの位置（小数） | `frameNo` |
| 再生ヘッドを動かす | `frameNo`（プロパティの形）/ `seek(frameNo)`（動詞の形） |
| パーツ名の一覧（パーツのインデックス順） | `partNames` |
| アニメーションの総フレーム数 | `totalFrames` |
| 保持する / 保持を解く | `pause` / `resume` |
| その場で再生を終える | `stop` |
| ホストのトランスフォームを動かさずに描画を反転する | `flipX` / `flipY` |
| そのフレームのパーツの合成行列 | `partTransform` |
| そのフレームでパーツが非表示か | `isPartHidden` |
| パーツがスキンメッシュか | `isPartSkinnedMesh` |

**`animationName` ではなく `currentAnimation`。** `animationName` は `animationNames` と 1 文字しか違わず、しかもこの 2 つはホストが組にして使います（一覧からピッカーを作り、選ばれているものを読み戻す）。Godot も `AnimationPlayer.current_animation` と呼んでいます。選ぶ操作は `setAnimation(name)` のままです。選択をプロパティで表すホストでは、そのプロパティが `currentAnimation` を名乗ります。

**`frame` ではなく `frameNo`。** この値は小数です（サブフレームでは再生ヘッドがフレームの間に来ます）。また `frame` は、描画フレームや 1 tick が生む FrameData を指す語として既に使われています。Godot の `AnimatedSprite2D.frame` は整数のインデックスなので、その綴りを借りると Godot の読み手に違う型を約束することになります。Brain 自身の名前も `frame_no` です。

再生ヘッドを動かす側は、冒頭のとおり「プロパティかメソッドか」をホストに任せます。プロパティで書くホストは同じ語幹への代入、動詞を好むホストは `seek` です。3 つめの語幹を作らないでください。

**`partNames` は必須です。パーツの個数は代わりになりません。** 個数では、イベントの `partIndex` を人が読める形に戻せず、パーツ一覧の UI も作れません。個数はリストの長さで得られます。

再生方向は `playbackDirection` と `playbackStyle` の 2 つの列挙型で表します。真偽値 2 つ（「逆再生か」「往復か」）にしないでください。1 回の呼び出しでまとめて設定できなくなります。

**列挙子の名前は Brain のものをそのまま使います** — `Forward` / `Backward` と `Normal` / `PingPong`。大文字小文字と区切りはホストに従いますが、語幹は変えません。`Reverse` は `Backward` の綴り違いではなく同じ概念の 2 つめの語で、他の Player のドキュメントを読んだ人はそれを見つけられません。接頭辞の要る言語では、`PLAY_*` と `STYLE_*` ではなく `DIRECTION_*` と `STYLE_*` で対にしてください。

オーバーライドの優先度も Brain の列挙子名をそのまま使います。`OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` / `Permanent`。

### 3-2. アニメーションミキシング

トラックスタックを公開する Player は、次の語幹を使ってください。いずれも C-API から採った名前です。

**トラック**は、その下のトラックが作ったポーズの上に重なります。**ソース**はトラックの中で混ぜる 1 本のアニメーションで、トラックの `main` ソースがその構造を決めます（[アニメーションミキシング](../50_features/10_animation_mixing.ja.md)）。

| 概念 | 語幹 |
|---|---|
| トラックの数 | `trackCount` |
| トラックにアニメーションを載せる | `setTrackAnimation` |
| トラックの main ソース | `trackMainSource` |
| トラックを空にする | `clearTrack` |
| 下のポーズにどれだけ重ねるか | `trackAlpha` |
| どう重ねるか（`Mix` / `Additive`） | `trackBlend` |
| トラックに載せたアニメーションをどこから始めるか（`Free` / `Start`） | `trackSync` |
| トラックの再生ヘッド | `trackFrame` |
| トラックの速度 | `trackSpeed` |
| トラックのフレームレートの上書き | `trackFrameRate` |
| トラックのフレームスキップのオン・オフ | `trackFrameSkipEnabled` |
| トラックの再生方向 / 往復のスタイル | `trackPlaybackDirection` / `trackPlaybackStyle` |
| トラックのアニメーションのループ回数 | `trackLoopCount` |
| トラックのアニメーションが規定回数を再生し終えたか（状態） | `isTrackFinished` |
| 直前の update でトラックのアニメーションがループしたか（パルス） | `trackJustLooped` |
| トラックのアニメーションを差し替えながらフェードする | `crossfade` |
| トラック自身のアルファをフェードする | `fadeTrack` |
| 進行中のフェードの進み具合 | `trackFadeProgress` |
| 進行中のフェードを止める | `cancelTrackFade` |
| トラックに別のアニメーションを足す | `addTrackSource` |
| トラックの中でのソースの重み | `sourceWeight` |
| ソースの再生ヘッド | `sourceFrame` |
| ソースがイベントを発火する重みの下限 | `sourceEventThreshold` |
| クロスフェードの出ていく側がイベントを発火する割合の下限 | `trackOutgoingEventThreshold` |

**`crossfade` と `fadeTrack` は別物です。** 前者はトラックが再生する内容を差し替え、後者はトラックがどれだけ効くかを変えます。

再生設定のトラック版は、§3-1 の語幹の前に `track` を付けます。track 0 の設定は §3-1 の語幹のままです（`speedScale` は track 0 の `trackSpeed`）。track 0 への `crossfade` と `setTrackAnimation` は、`setAnimation` と同じく、呼んだ時点から `currentAnimation` を替えます。

読みと書きは §3 のとおり語幹を共有します（`trackAlpha` は両方に使い、書き込みの形はホストに従います）。`setTrackAnimation` が動詞のままなのは、対になる読みが無いからです。返るのはソースのハンドルで、それは `trackMainSource` で読めます。

### 3-3. 用語

**`cellmap` は 1 語です。** `cellMap` や `CellMap` と綴らないでください。スキーマとこのガイドは `cellmap` で統一しています。

---

## 4. イベント

Player は 6 つのイベントを公開します。**届け方**はホストの作法に任せますが、**名前**と**どれを持つか**は任せません。

### 4-1. 語幹

| 何を伝えるか | 語幹 |
|---|---|
| **ユーザーデータ**のキーを通過した | `userData` |
| **シグナル**のキーを通過した | `signal` |
| **オーディオ**のキーを通過した | `audio` |
| `play` で再生を始めた | `animationStarted` |
| 設定したループをすべて再生し終えた | `animationFinished` |
| 再生ヘッドがループ境界を跨いだ | `animationLooped` |

`on` 接頭辞・大文字小文字・区切りはホストに従います。Godot のシグナルなら `animation_finished`、C# の event なら `OnAnimationFinished`、TypeScript の購読なら `onAnimationFinished`、Python のコンストラクタ引数なら `on_animation_finished` です。揃えるのは語幹の**全体**です。`animationFinished` を `finished` に縮めるのは別の名前になります。エフェクト・インスタンス・音声・テクスチャを持つオブジェクトの上で、`finished` だけでは何が終わったのか分かりません。

後ろの 3 つは、§3-1 の語彙表で名前の決まっている概念です（`justStarted` / `isFinished` / `justLooped`）。**対になる名前は綴りを揃えてください。** `isFinished` の隣に `onComplete` や `onAnimationEnd` を並べると、1 つの概念を 2 つの語で呼ぶことになります。

### 4-2. 持つべき集合

上の 6 つは必須です。1 つでも欠けると、他の Player で観測できることがその Player では観測できなくなります。

次の 3 つは、持っている Player もありますが、必須ではありません。

| 必須でないもの | 持つ価値がある場面 |
|---|---|
| アニメーション切り替えのイベント | `setAnimation` を呼んだのはホスト自身なので、既に知っています。インスペクタのフィールドや宣言的な prop など、**別の入り口**でアニメーションを選べる Player だけが公開し、その理由をそばに書いてください |
| 毎フレームのフック | シーングラフを持つ Player でだけ意味があります。パーツ追従をつなぐ場所で、そのフレームの行列が決まった後に呼びます |
| 生のイベント列 | デコード済みの 3 つを迂回する手段。デコードが重い場合や、ホストがインデックスそのものを欲しい場合に役立ちます。デコード済みのイベントの代わりにはなりません |

### 4-3. 届け方はホストに任せる

Godot のシグナル、Unity の `event` と `UnityEvent`、TypeScript の「購読すると解除関数が返る」形、Dart や React の代入するコールバック、Ren'Py のホストが吸い出すキュー、Rust のポーリングするスライス。どれもそのホストで正しい形です。統一しないでください。

ただし、次の 2 つを守ってください。

**登録できるリスナーの数を明記してください。** 1 つか複数かはホストに任せます（Flutter の widget コールバックや React の prop は 1 つ、Godot のシグナルや C# の event は複数）。呼び出し側は、2 つめを足せると思っていたら、1 つめが黙って呼ばれなくなって初めて気づきます。どちらにしたかを書いてください。

**コールバックの仕組みを持たない Player は、パルスを公開してください。** ホストが購読せずポーリングする場合、完了の立ち上がりを取る手段が他にありません。その場合は `justStarted` と `justFinished` を `justLooped` と並べて公開します。

---

## 5. tick の単位はホストのクロックに従う

ホストが駆動する Player は経過時間を受け取ります。**その単位はホストのクロックに従ってください。** エンジンが秒を渡すなら秒（`_process(delta)`、`Time.deltaTime`）、ミリ秒を渡すならミリ秒（`requestAnimationFrame`）、`Duration` を持つホストなら `Duration` です。入り口で変換すると、1000 倍の変換が隠れる場所が 1 つ増えるだけです。

**シグネチャを見て単位が分かるようにしてください。** `deltaMs` / `dt_ms` / `delta_secs` のような引数名にするか、単位を型に持たせます。裸の `delta` でよいのは、ホスト自身のクロックが単位を決めている場合だけです（Godot の `delta` と Unity の `deltaTime` は秒以外にありえません）。

動詞もホストに任せます。`update` と `advance` のどちらを使ってもかまいません。

---

## 6. 既定値

同じ機能の既定値が Player ごとに違うと、移植したときに挙動が黙って変わります。次の値は Player が決めてよいものではありません。

| 対象 | 既定 | 理由 |
|---|---|---|
| オーバーライドの `priority` | `HoldUntilNextAnimation` | C-API の既定 |
| 表示オーバーライドの `cascade` | `false` | SpriteStudio のキーフレームの非表示は子孫に及ばないので、波及は明示したときだけ |
| カラーオーバーライドの `blendOp` | `Mix` | 零値。色のアルファが強度になるので「素のまま」を意味する |
| パーツ追従の、パーツが非表示のときの挙動 | 追従を続ける | パーツが 1 フレーム隠れただけで追従対象を消さない |
| アニメーションを 1 つもセットアップしていないときの `isPlayingForward` | `true` | 前進が静止状態。偽を返すと、ホストの音声が最初のフレームで止められる。セットアップの後は、停止中も含めて実際の進行方向を返す（往復の復路で止めたときや `Backward` を設定したときは偽。[再生制御 §5-3](../30_core/20_playback_control.ja.md#5-3-再生方向)） |
| ホストが設定した `loopCount` | `setAnimation` と track 0 の切り替えをまたいで保つ | ホストは 1 回設定すれば済む。Brain のセットアップは既定値に戻すので、Player が設定し直す（§8-1） |

`subFrameEnabled` と `frameSkipEnabled` の既定は、ホストのエンジンの慣習に合わせてかまいません。

配置（`fit` / `align`）には独自の既定値があり、契約の残りと一緒に[配置：fit と align](../50_features/70_placement.ja.md)にまとめています。こちらも Player が選ぶものではありません。

### 6-1. パーツ追従のオプション

パーツ追従（アタッチメント / トラッカー / アンカー）を持つ Player は、同じオプションを同じ既定値で提供してください。

| オプション | 既定 | 理由 |
|---|---|---|
| position | `true` | 位置を追わない追従は追従ではない |
| rotation | `true` | 手に持たせた武器は手と一緒に回る |
| scale | **`false`** | 追従対象はパーツの**位置に置く**もので、パーツに**大きさを決めさせる**ものではない。頭上の HP バーやネームプレートが、パーツの拡縮アニメーションで伸び縮みするのはたいてい望まれない |
| 非表示のとき | 追従を続ける | パーツが 1 フレーム隠れただけで追従対象を消さない |

**接頭辞はホストの作法に任せます。** 揃えるのは 3 つの組と既定値で、綴りではありません。裸の `scale` がホストのトランスフォーム自身の拡縮と読み違えられる言語では、`drive` や `update` などの接頭辞を付けてかまいません（Godot が `update_position` などを使うのは `RemoteTransform2D` にならっているためです）。

---

## 7. 失敗の伝え方

**どの仕組みで失敗を伝えるかはホストの作法に任せます。何を伝えるかは任せません。**

`false`、`null`、例外、`Result`、`Option` — どれもその言語で正しい形です。揃えるのは、失敗が**読める**ことです。呼び出し側が「自分は何を要求したのか」「代わりに何があったのか」、つまり「次に何を書けばよいのか」を判断できるようにしてください。

### 7-1. 検索は黙って失敗してよい。命令は黙って失敗してはいけない

| 形 | 失敗したとき |
|---|---|
| `find…` — 失敗しうる検索 | 戻り値だけで伝える |
| `setAnimation` / `load…` / `set…Override` — 何もしなかった命令 | 戻り値**と**報告で伝える |

`findPartIndex` が `-1` を返すのは「このパーツはあるか」への完全な答えです。一方、`setAnimation` が `false` を返しても、呼び出し側は効くと信じて命令を出しています。症状は「前のアニメーションを再生し続ける」か「何も描かない」です。**読まなくてもよい bool は報告になりません。**

### 7-2. 報告に載せるもの

**必須なのは、呼び出し側が他の手段で得られないものだけです。** どれも、失敗した呼び出しが既に持っている情報です。

| 呼び出し | 報告に書くもの |
|---|---|
| `setAnimation` | パックと、渡された名前 |
| パックの読み込み | 渡された URL / パス / アセットキーと、**下位の原因をそのまま** |
| テクスチャの解決 | 探したファイルと、それが属するセルマップ |
| `setPartCellOverride` | パックと、渡されたセルマップ名・セル名 |
| パーツ名を取るオーバーライド | 渡されたパーツ名 |

読み込みの行に原因が入っているのは、他に運ぶものが無いからです。版の不一致は「期待した版」と「届いた版」を知っており、報告がその数字を落とすと誰にも分からなくなります。

**「代わりに何があったか」の一覧は必須ではありません。** `animationNames` / `partNames` / `cellmapNames` / `cellNames` は §3-1 の必須の語彙なので、一覧が欲しい呼び出し側は自分で出せます。これを必須にすると、アニメーションが 200 本あるパックのために、すべての Player が打ち切りの規則を持つことになります。短くて明らかに役立つ場面では添えてかまいません。

**呼び出し側が書いたものを名指ししてください。** 呼び出し側が書いたのは `"wakl"` であって `0x4f3a21bc` ではありません。名前のハッシュは Brain 側の識別子で、名前ではありません。パーツ名を取る呼び出しの報告にパーツのインデックスを出すのも、読み込みの失敗で原因だけを出して渡されたパスを落とすのも同じ誤りです。

### 7-3. 1 回だけ、1 層だけ

**1 回だけ報告してください。** これらの呼び出しは毎フレーム通る経路に乗ります。ステートマシンはクリップを毎フレーム指定し直しますが、失敗した名前は「セットアップ済み」にならないので、同じ名前なら何もしないガード（§8-2）に守られず、毎フレーム失敗の分岐を通ります。回数を抑えないと、1 つの綴り間違いがフレームレートの速さでログを埋めます。抑えるキーは呼び出し箇所ではなく失敗そのもの（種別と名前の組など）にしてください。2 つめの間違った名前は、別の行として報告されます。

**1 層だけで報告してください。** 公開層の下に内部の Player を持つ Player では、同じ行を書ける場所が 2 つあります。ホストが呼んだ層で報告してください。内側の層の報告には、呼び出し側が触れない型の名前が出てしまいます。

### 7-4. 出力先と強さはホストに任せる

Godot の Output パネル、Unity の Console、ブラウザのコンソール、`FlutterError`、Ren'Py のログ。呼び出し側が明示的につないだ構造化されたエラーの経路がホストにあれば（`onError` プロパティ、reject される `Future`、消費が必須の `Result`）、そちらが本線で、ログはどちらもつながなかった呼び出し側のための受け皿です。

強さは影響の大きさで決めます。読み込めなかったパックは Player に何も描かせないのでエラー、効かなかった `setAnimation` は前のクリップを描き続けるだけなので警告です。

---

## 8. アニメーションの選択と再生

C-API の再生制御の意味は [再生制御](../30_core/20_playback_control.ja.md) にあります。ここでは、それを Player の公開 API にどう出すかを決めます。

### 8-1. 選択と再生を分ける

**`setAnimation(name)` と `play()` を別のメソッドとして公開してください。** `setAnimation` は `ss_runtime_setup_animation` を、`play` は `ss_runtime_play`（または `ss_runtime_play_with_start_frame`）を呼びます。

| API | 内容 |
|---|---|
| `setAnimation(name)` | アニメーションを選び、先頭フレームで待機する。**再生は始めない**。Brain の内部でメモリを確保する |
| `play()` / `play(startFrame)` | 選んだアニメーションを先頭（または指定フレーム）から再生する。メモリを確保しない |
| `pause()` / `resume()` | その場で保持する / 保持を解いて続きから再生する |
| `stop()` | その場で再生を終える |

分ける理由は 3 つあります。

- `ss_runtime_setup_animation` はメモリを確保します。`play` の中で呼ぶと、再生のたびに確保が走ります。
- 再生せずに先頭フレームのポーズを表示できます。
- 重い準備（セットアップや、そのアニメーションが使うアセットの読み込み）を `play` の前に済ませ、再生をすぐに始められます。

**`resume` を `play` にまとめないでください。** `play` は常に先頭から始めるので、一時停止の解除に使うと再生ヘッドが先頭に戻ります。**`isPlaying` と並べて `isPausing` を公開してください。** 一時停止中も `isPlaying` は真のままです。

再生区間は `ss_runtime_setup_animation` が既定値に戻すので、`setAnimation` の中か、`play` の前に設定してください。

**ループ回数は、ホストが設定した値を Player が保ってください。** `ss_runtime_setup_animation` はループ回数も既定値に戻しますが、Player の公開 API では、ホストが設定した `loopCount` は `setAnimation` をまたいで残ります。Player はホストが最後に設定した値を持っておき、`setAnimation` の後と、track 0 のアニメーションを替える操作（track 0 への `crossfade` と `setTrackAnimation`）の後に、`ss_runtime_set_loop_count` で設定し直してください。

速度・フレームレートの上書き・フレームスキップ・再生方向はセットアップをまたいで保持されるので、設定し直す必要はありません（[再生制御 §3](../30_core/20_playback_control.ja.md#3-セットアップが戻す設定)）。

**ラベルで指定した区間も同じ状態です。** `setAnimationSectionByLabel(startLabel, endLabel, startOffset = 0, endOffset = 0)` は、アニメーションのラベル表から各ラベルのフレームを求め、`setAnimationSection` と同じく区間を設定します。再生はしません。区間は、開始ラベルのフレームに `startOffset` を足した位置から、終了ラベルのフレームに `endOffset` を足した位置までで、インスタンスパーツの開始・終了ラベルと同じ規則です。ほかの区間と同じく、`setAnimation` の中か、`play` の前に設定してください。

### 8-2. 選択済みのアニメーションを指定されたら何もしない

**`setAnimation(name)` は、`name` が既にセットアップ済みのアニメーションなら、C-API を呼ばずに戻ってください。**

「選ぶ」操作なので、選び直すものが無ければ何もしません。一方で `ss_runtime_setup_animation` は何もしない呼び出しではありません。再生を Stopped に戻し、再生ヘッドを巻き戻し、再生区間を捨て、`Permanent` 以外のオーバーライドをすべて解除します。現在のクリップを毎フレーム指定するステートマシンは、ごく普通のホストのコードです。

```cpp
// ガードが無いと、このアニメーションは永久に再生されない
player.setAnimation(state.clip);
```

ガードはもう 1 つの不具合も防ぎます。`ss_runtime_set_loop_count` は書き込むたびに残りのループ回数を設定し直すので、セットアップのたびにループ回数を設定する Player は、毎フレームのセットアップでループ回数も毎フレーム戻し、有限のループが終わらなくなります。ガードはその設定より前に置いてください。

**「セットアップ済み」は Brain が持っている状態と比べ、リソースをバインドしたら必ず無効にしてください。**

- ホストが最後に要求した名前と比べてはいけません。アニメーション名をシリアライズして持つコンポーネントは、最初のセットアップより前からその値を持っているので、最初の `setAnimation` が飛ばされ、何もセットアップされません。
- `ss_runtime_bind_resource`（とそれを呼ぶ読み込み処理）の後は、何もセットアップされていません。バインドで「セットアップ済み」を消してください。消さないと、前のパックと同じ名前のアニメーションを持つ次のパックで最初の `setAnimation` が飛ばされ、そのパックが再生できなくなります。1 つのプロジェクトから書き出したキャラクター一式では、名前が重なるのが普通です。

アニメーションを先頭からやり直すには `play` を、オーバーライドを捨てるには clear-overrides の API を、再生区間を捨てるにはアニメーション全体を覆う区間を設定します。

公式の Player 7 本はすべてこの挙動です。C-API を包む内部の層は、C-API の名前と意味をそのまま保ちます（`SpriteStudioInternalPlayer.SetupAnimation`、`SsInternalPlayer.setupAnimation` など）。ガードは、その上の利用者に向いた層に置きます。

---

## 9. パーツの読み出し

`partTransform`・`isPartHidden` など、パーツ 1 つを読む API は、**直前の tick が組み立てたフレーム**から答えます。その tick の `ss_runtime_get_frame_data` が返した FrameData とワールド行列で、次の呼び出しまで有効です（[描画ループ §1](../30_core/40_render_loop.ja.md#1-soa-バッファ)）。**読み出しでフレームを組み立てることはしません。**

| ホストが読むとき | 返るもの |
|---|---|
| tick の後 | その tick のフレーム。描画しているフレームです |
| オーバーライドを設定・解除した後、次の tick の前 | 直前の tick のフレーム。オーバーライドは次の tick から反映されます |
| tick を挟まずにもう一度 | 同じ答え |
| フレームを組み立てなかった tick（画面外のキャラクターを Player が飛ばした tick など）の後 | 最後に組み立てたフレーム。パーツの姿勢と非表示が対のまま揃います |
| 最初の tick の前 | `isPartHidden` は、存在するパーツについて「非表示ではない」と答えます。「非表示」と答えると、まだ何も描いていないうちから、呼び出し側が非表示の分岐に入ってしまいます |

読み出しのたびにフレームを組み立てると、1 回の呼び出しに 1 フレーム分のコストがかかります。パーツ追従は毎 tick、追従するもの 1 つにつき 1 回読むので、そのコストが掛け算になります。しかも、次の tick が組み立てる結果まで変わります。`OverwriteOnNextKeyframe` のオーバーライドが、設定したフレームのキーを数えないのは、最初に評価された update の中だけです（[パーツオーバーライド §1-3](../50_features/20_part_overrides.ja.md#1-3-優先度モード)）。`play` やシークの後の update は開始フレームを通過するので、そこで設定したオーバーライドは、その update で最初に評価される必要があります。update の前に読み出しがフレームを組み立てると、最初の評価をそちらが取ってしまい、update はオーバーライドをアニメーションに返します。

---

## 10. この規約から外れるとき

ホストの言語やエンジンの慣習と正面から衝突する場合は、外れてかまいません。その場合は、**コードのそばに理由を書いてください。** 理由の書かれていない逸脱は、次に読む人には規約違反と区別がつきません。

Unity が座標系に Y-Up を渡し、他の Player が Y-Down を渡しているのは、正当な逸脱の例です。エンジンの座標系がそうなっているからで、その理由がコードに書かれています。

---
[目次に戻る](../README.ja.md)
