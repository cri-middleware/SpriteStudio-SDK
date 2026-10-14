> **日本語** | [English](./10_animation_mixing.md)

# アニメーションミキシング

複数のアニメーションを同時に再生する機能です。歩きに足を引きずる動きを重ねる、トラッキングの値で首を振らせる、待機から攻撃へなめらかに切り替える、といったことを、アニメーションを **トラック** に積んで実現します。ミキシングは Brain の状態と C-API だけの機能で、ssab を変える必要はありません。Player の公開 API での名前は [Player API 規約 §3-2](../20_design/30_api_conventions.ja.md#3-2-アニメーションミキシング) にあります。

## 1. モデル

Context はトラックの積み重ねを持ちます。トラックは番号の小さい順に、**セットアップポーズ**（ssae が定義する素の姿勢）の上へ重なります。

```
結果 = セットアップポーズ
track 0  →  重ねる   （主のアニメーション）
track 1  →  重ねる   （上に重ねるアニメーション）
track 2  →  重ねる   （さらにもう 1 つ）
```

調整の値は 2 つあり、**どちらを使うかが大事**です。

| | 役割 | 単位 |
|---|---|---|
| **`alpha`** | そのトラックを、下の結果にどれだけ重ねるか。**強さの調整はこちら** | トラックごと |
| **`weight`** | **同じトラック**に載った複数のアニメーションの比率 | ソースごと |

トラックの `alpha` は常に期待どおりに効きます。ソースの `weight` は、それだけでは期待どおりになりません。あるアトリビュートにキーを打っているアニメーションが 1 本だけなら、比べる相手がいないので、`weight` がどれだけ小さくても、そのアトリビュートはそのアニメーションの値になります。**`weight` はアニメーションどうしを本当に混ぜるとき（歩きと走り）だけに使い、それ以外は `alpha` を使ってください。**

### 1-1. track 0 は特別

track 0 が **構造** を受け持ちます。離散的なアトリビュート（セル・パーツカラー・シェーダ・反転・非表示・描画優先度・テクスチャ）、既定のイベントの発火、再生の状態（`is_playing`・`is_finished`・フレーム）は、すべて track 0 の主アニメーション（**主役**）から来ます。`ss_runtime_setup_animation` は「このアニメーションを track 0 に載せる」操作です。track 0 は空にできないので、消したいときは `alpha` をフェードしてください。

主役が替わるのは、track 0 のアニメーションを替えたときだけです。`ss_runtime_setup_animation`、track 0 への `ss_runtime_set_track_animation`、track 0 への `ss_runtime_crossfade` の 3 つで、`ss_runtime_crossfade` は**フェードの開始時に**主役を渡します（§4）。主役が再生しているアニメーションは、`ss_runtime_get_current_animation_index` で取れます（ssab の `animations()` の添字）。

### 1-2. トラックは自分がキーを打ったアトリビュートにだけ効く

トラックのアニメーションがキーを持たないアトリビュートは、下の結果がそのまま通ります。頭だけにキーを打ったアニメーションを重ねても、**マスクを指定しなくても**脚には影響しません。逆に言えば、**重ねたアニメーションが効く範囲は、作画した範囲そのもの**です。体の一部だけに効かせたいなら、その部分だけにキーを打ってください。

### 1-3. 同じ ssab の中だけ

1 つのリソース（1 つの ssab）の中のアニメーションだけを積めます。

### 1-4. キャラクター全体で 1 つの時計

すべてのトラックは、キャラクターの 1 つの時計で動きます。トラックに載せたアニメーションは載せた時点から再生され、`ss_runtime_update` がすべてのトラックとすべてのフェードをまとめて進めます。再生の操作はこの時計に対して働きます（[再生制御 §4](../30_core/20_playback_control.ja.md#4-再生の状態)）。

| 呼び出し | スタックへの効果 |
|---|---|
| `ss_runtime_play` | **track 0** を再生区間の先頭へ戻し、時計を動かします。他のトラックは再生位置をそのまま保ちます |
| `ss_runtime_pause` / `ss_runtime_resume` | すべてのトラックとすべてのフェードを止める／再開します |
| `ss_runtime_stop` | すべてのトラックとすべてのフェードをその場で止めます。再開できるのは `play` だけです |

- **track 0 が終わっても時計は止まりません。** track 0 で有限のループ回数を使い切っても、他のトラックとフェードは進み続けます。`is_playing`・`is_pausing`・`is_finished` は track 0 の状態を表すので、3 つとも通常どおり終了を報告します。終了後の `pause` は他のトラックを止め、`resume` で再開します。
- フェードは時計と一緒に進みます。一時停止中や停止中は、フェードも進みません。
- **時計には専用のフラグが 2 つあります。** `ss_runtime_is_clock_playing` は `play` から `stop` または次のセットアップまで `true`、`ss_runtime_is_clock_pausing` は `pause` で保持している間 `true` です。`is_playing` や `is_pausing` と違い、どちらも track 0 が終わった後も時計の状態を返します。動くものが無いキャラクターの tick を止める Player は、再生の操作を自前で写さず、これを読んでください。写しは 1 つ取りこぼすと食い違います。セットアップでも時計は止まります。
- **track 0 以外に動くものがあるかも返せます。** `ss_runtime_is_stack_in_motion` は、いずれかのトラックでフェードが進んでいる間、または聞こえるトラックで track 0 のメイン以外のソースが 0 より大きい速度と重みで再生中の間、`true` を返します。クリアした・終わった・無音にした・速度 0 で手動操作しているトラックは数えません。途中で取り消したクロスフェードの送り出し側は数えます。時計は読みません。track 0 のフレームが変わらない間は描き直しを省く Player は、これが `true` で、かつ時計が保持されずに動いている（`ss_runtime_is_clock_playing` が真で `ss_runtime_is_clock_pausing` が偽）ときは描き直してください。積んだトラックやソースを自前で写す必要はありません。クロスフェードが終わったときのように、runtime が自分で解放するものがあるためです。

### 1-5. トラックの再生設定

速度・フレームレートの上書き・フレームスキップ・再生方向は、そこに載っているアニメーションではなく、**トラック**の設定です。設定は、そのトラックにいま載っているすべてのアニメーション（クロスフェード中なら両側）と、あとから載せるすべてのアニメーションに効きます。クロスフェードや `ss_runtime_set_track_animation` で元に戻ることはありません。

| 設定 | 任意のトラック | track 0（従来の名前） |
|---|---|---|
| 速度 | `ss_runtime_set_track_speed` | `ss_runtime_set_speed_scale` |
| フレームレートの上書き | `ss_runtime_set_track_frame_rate` | `ss_runtime_set_frame_rate` |
| フレームスキップ | `ss_runtime_set_track_skip_frames` | `ss_runtime_set_skip_frames` |
| 再生方向とスタイル | `ss_runtime_set_track_playback_direction` | `ss_runtime_set_playback_direction` |
| ループ回数 | `ss_runtime_set_track_loop_count` | `ss_runtime_set_loop_count` |

- **ループ回数**と**再生区間**は例外で、アニメーションの設定です。そのアニメーションのフレームを数えたり番号で指したりするものだからです。アニメーションを載せると、区間は全体に、ループは無限に戻るので、回数は載せた後で指定してください。ホストのループ回数を覚えている Player は、`ss_runtime_setup_animation` の後と同じく、クロスフェードを含めて track 0 のアニメーションが替わるたびに、それをかけ直します。
- track 0 の設定は `ss_runtime_setup_animation` を越えて残り、`ss_runtime_reset` で消えます。他のトラックの設定は、`ss_runtime_setup_animation` がスタックをたたむときに、トラックごと消えます。
- まだ何も載っていないトラックにも設定できます。そのトラックはこのために作られます。ループ回数だけは、トラックにアニメーションが載っている必要があります。
- track 1 以降も、自分の終了とループを報告します。`ss_runtime_is_track_finished` と `ss_runtime_track_just_looped` で、track 0 の `is_finished` と `just_looped` にあたります。終わったトラックは**最後のフレームで止まり**、そのポーズを混ぜ続けます。フェードやクリアをいつするかはホストが決めます。キャラクターの再生を終わらせるのは、track 0 の終了だけです。

## 2. 1 本のアニメーションを再生する

ミキシングを使わない場合は、これまでどおりです。

```cpp
ss_runtime_setup_animation(ctx, "walk");   // = track 0 に載せる
ss_runtime_update(ctx, delta_ms);
ss_runtime_get_frame_data(ctx, frame_no, &fd, &fd_len);
```

## 3. 重ねる

```cpp
ss_runtime_setup_animation(ctx, "idle");            // track 0: 主の演技

ss_runtime_set_track_animation(ctx, 1, "look_around");
ss_runtime_set_track_alpha(ctx, 1, 0.4f);           // 40% 重ねる
```

トラックは必要に応じて作られます（track 3 を設定すると、track 1・2 も空のまま作られます）。`alpha` は 0〜1 の範囲に制限しません。1 を超えると外挿になり、重ねる用途ではそれが欲しい場合があります。

### 3-1. 加算のトラック

`SS_RUNTIME_TRACK_BLEND_ADDITIVE` のトラックは、そのトラックの姿勢へ近づける代わりに、**セットアップポーズからの差分**を足します。下のアニメーションが残り続けます。

```cpp
ss_runtime_set_track_animation(ctx, 1, "lean_left");
ss_runtime_set_track_blend(ctx, 1, SS_RUNTIME_TRACK_BLEND_ADDITIVE);
ss_runtime_set_track_alpha(ctx, 1, 0.6f);           // 歩きの上に、傾きを 60%
```

連続した値で動かす軸にはこれを使います。軸のアニメーションは**中央のフレームがセットアップポーズと同じ**になるように作り、再生するか、フレームを直接動かしてください。

```cpp
// 再生せず、トラッキングの値でフレームを動かす
ss_runtime_set_track_frame(ctx, 1, 30.0f + yaw * 30.0f);
```

## 4. 切り替える

`ss_runtime_crossfade` は、トラックがいま再生しているアニメーションから、新しいアニメーションへフェードします。ハンドルを組で管理する必要も、後始末も要りません。

```cpp
ss_runtime_crossfade(ctx, 0, "run", 300.0f);   // 300ms。update の経過時間と同じ単位

ss_runtime_update(ctx, delta_ms);
float progress = ss_runtime_get_track_fade_progress(ctx, 0);   // 0〜1。フェード中でなければ NaN
```

入ってくるアニメーションは、**呼んだ時点から**そのトラックの主アニメーションになります。呼んだ直後の `ss_runtime_get_track_main_source` は、入ってくる側を返します。track 0 ではその時点から主役になる（§1-1）ので、セルなどの離散的なアトリビュートはフェードの最初のフレームから入ってくる側のものになり、再生の状態とフレームも入ってくる側のものになります。フェード中に出ていく側が終わっても、キャラクターは終了になりません。入ってくるアニメーションは、トラックの設定（§1-5）で再生されます。

トラックにすでに載っていたものは、すべてまとめて**出ていく側**としてフェードアウトします。ソースどうしの比率は保ったまま、トラックの `1 - t` の割合に縮み、入ってくる側が `t` を受け持ちます。フェードが終わると、出ていく側のソースはすべて解放されます。

| 状況 | 挙動 |
|---|---|
| `duration_ms <= 0` / `NaN` / 無いアニメーション名 | `false` を返す。理由は `ss_runtime_get_last_error` |
| 何も再生していないトラックへの `crossfade` | フェードせず、すぐに再生を始める |
| フェード中のトラックへもう一度 `crossfade` | 進行中のフェードを**その時点の状態のまま**引き取る。両側ともその時点の weight で新しい出ていく側に入るので、姿勢は飛ばない |
| 複数のソースが載ったトラックへの `crossfade` | すべてまとめてフェードアウトする |
| フェードを止めたい | `ss_runtime_cancel_track_fade`。`alpha` と `weight` はその時点の値で止まる。残ったものは次の `crossfade` で一緒にフェードアウトする |

### 4-1. セットアップポーズへ戻す

積み重ねの一番下はセットアップポーズなので、トラックをフェードアウトさせると、そのままセットアップポーズへ戻ります。

```cpp
ss_runtime_fade_track(ctx, 0, 0.0f, 200.0f);   // 200ms かけてセットアップポーズへ
```

## 5. アニメーションどうしを混ぜる

比率で混ぜる場合（歩きと走りの合成など）は、同じトラックに載せて `weight` を設定します。

```cpp
ss_runtime_setup_animation(ctx, "walk");
uint32_t walk = ss_runtime_get_track_main_source(ctx, 0);
uint32_t run  = ss_runtime_add_track_source(ctx, 0, "run", 0.0f);

// 歩き 70% / 走り 30%
ss_runtime_set_source_weight(ctx, walk, 0.7f);
ss_runtime_set_source_weight(ctx, run, 0.3f);
ss_runtime_set_source_frame(ctx, run, 0.0f);
```

`ss_runtime_get_track_main_source` は、`ss_runtime_set_track_animation`・`ss_runtime_crossfade`（か `ss_runtime_setup_animation`）がそのトラックに載せたアニメーションのハンドルを返します。`weight` は 1 から始まります。

`weight` は絶対値ではなく比率です（`(0.7, 0.3)` と `(7, 3)` は同じ結果）。ソースはそれぞれ自分の再生ヘッドを持ち、`ss_runtime_update` がすべてを進めます。**尺の違うソースの位相はそろいません。** track 0 と同じ位相から始めるにはトラックの同期（§5-1）を使います。そろえ続けたい場合は、Player がシークしてください。

### 5-1. track 0 と同じ位相から始める

トラックに載せたアニメーションは、再生区間の先頭から始まります。歩幅の途中にいる歩きの上に走りを載せると、足の運びが食い違います。トラックの同期を `SS_RUNTIME_TRACK_SYNC_START` にすると、そのあと載せたアニメーションは、track 0 のアニメーションがいる**位相**から始まります。位相とは、再生区間のどこまで進んだかを、区間の先頭を `0.0`、終端に向かって `1.0` とした割合で表したものです。尺の違うアニメーションも、割合でそろいます。

```cpp
ss_runtime_set_track_sync(ctx, 0, SS_RUNTIME_TRACK_SYNC_START);
ss_runtime_crossfade(ctx, 0, "run", 300.0f);   // 走りは歩きと同じ位相から始まる
```

| | `SS_RUNTIME_TRACK_SYNC_FREE`（既定） | `SS_RUNTIME_TRACK_SYNC_START` |
|---|---|---|
| 載せたアニメーションの開始位置 | 再生区間の先頭 | track 0 のアニメーションと同じ、再生区間の中の割合の位置 |
| 向いている使い方 | 一回きりの動き（まばたき、口の形、攻撃など） | 周期的な動きどうしを重ねる（歩きと走りなど） |

- そのあとトラックに載せるすべての操作に効きます。`ss_runtime_set_track_animation`、`ss_runtime_crossfade`（track 0 でも効き、入ってくるアニメーションは出ていくアニメーションの位相から始まる）、`ss_runtime_add_track_source` です。**載せる前に設定してください。** まだ何も載っていないトラックは、このために作られます。
- 基準は track 0 の主アニメーションで、track 0 のクロスフェードは開始時にそれを渡します。入ってくるアニメーションは渡す前に載せるので、出ていくアニメーションの位相から始まります。
- 位相は、進行方向ではなく再生区間の中の位置で読みます。逆再生や往復再生の戻りの途中でも、その位置から相手のアニメーションを始めます。
- そろえるのは、アニメーションを載せたときの 1 回だけです。その後は両方が同じ時計で進むので、尺と速度が同じなら位相はそろったままです。尺や速度が違えば、またずれていきます。
- ループの扱いは変わりません。途中から始めたアニメーションも、終端に達すれば通常どおり先頭に戻ります。
- トラックの alpha や blend と同じく、トラックの設定です。`ss_runtime_setup_animation` がスタックをたたむと、どのトラックも `FREE` に戻ります。

## 6. イベント

既定では、track 0 のアニメーションだけがイベントを発火します。他のソースも発火させるには、そのソースの閾値を、その `weight` 以下に下げます。`weight >= 閾値` のソースのイベントが報告されます。閾値の既定値は `+∞` で、どの weight も届かないため、他のトラックに1つだけ載せたソースも weight 1 では発火しません。

```cpp
ss_runtime_set_source_event_threshold(ctx, run, 0.5f);   // run の weight が 0.5 以上なら発火する
```

クロスフェードの出ていく側は例外です。出ていく側のソースは、その側がトラックに占める割合 `1 - t` が**トラックの**出ていく側の閾値以上のあいだ発火します。ソースが何本あっても同じで、フェード中は各ソース自身の閾値は見ません。既定は `+∞` なので、出ていく側はフェードの開始と同時に発火しなくなります。track 0 では、ちょうど入ってくる側が発火を始める時点です。`0.5` にすると、フェードの前半のあいだ発火します。トラックの設定なので、そのトラックのすべてのクロスフェードに効きます。track 0 の値は `ss_runtime_setup_animation` を越えて残ります。

```cpp
ss_runtime_set_track_outgoing_event_threshold(ctx, 0, 0.5f);   // 出ていく側はフェードの前半だけ発火する
```

イベントには、発火したソースのハンドルが付くので、どのソースのイベントかを区別できます。イベントの読み方は [イベント §4](../30_core/30_events.ja.md#4-複数のアニメーションを重ねているとき) にあります。

```cpp
int n = ss_runtime_get_passed_event_count(ctx);
for (int i = 0; i < n; ++i) {
    int32_t  event_idx = ss_runtime_get_passed_event_index(ctx, i);
    uint32_t source    = ss_runtime_get_passed_event_source_handle(ctx, i);
}
```

## 7. C-API の一覧

失敗の理由は `ss_runtime_get_last_error(ctx)` で取れます。`float` を返す関数が失敗のしるしに `NaN` を使うのは、正しい値の `0.0` と「そのトラックやソースは無い」を区別するためです。C では `isnan`、JavaScript では `Number.isNaN` で判定してください。整数を返す関数は、設定として読み戻されることのない値を失敗のしるしに使います。

### 7-1. トラック

| 関数 | 戻り値 | 失敗したとき |
|---|---|---|
| `ss_runtime_set_track_animation(ctx, track, name)` | `bool` | `false` |
| `ss_runtime_set_track_animation_by_hash(ctx, track, hash)` | `bool` | `false` |
| `ss_runtime_set_track_alpha(ctx, track, alpha)` | `bool` | `false` |
| `ss_runtime_get_track_alpha(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_blend(ctx, track, blend)` | `bool` | `false` |
| `ss_runtime_get_track_blend(ctx, track)` | `uint32_t`（`0` = Mix / `1` = Additive） | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_set_track_sync(ctx, track, sync)` | `bool` | `false` |
| `ss_runtime_get_track_sync(ctx, track)` | `uint32_t`（`0` = Free / `1` = Start） | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_get_track_count(ctx)` | `uint32_t` | `0` |
| `ss_runtime_clear_track(ctx, track)` | `bool` | `false`（track 0 は空にできない） |
| `ss_runtime_set_track_frame(ctx, track, frame)` | `bool` | `false` |
| `ss_runtime_get_track_frame(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_speed(ctx, track, speed)` | `bool` | `false` |
| `ss_runtime_get_track_speed(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_frame_rate(ctx, track, fps)` | `bool` | `false` |
| `ss_runtime_get_track_frame_rate(ctx, track)` | `int32_t`（アニメーションも上書きも無ければ `0`） | `-1` |
| `ss_runtime_set_track_skip_frames(ctx, track, skip)` | `bool` | `false` |
| `ss_runtime_get_track_skip_frames(ctx, track)` | `bool` | `false` |
| `ss_runtime_set_track_playback_direction(ctx, track, direction, style)` | `bool` | `false` |
| `ss_runtime_get_track_playback_direction(ctx, track)` | `int32_t`（`0` = 順方向 / `1` = 逆方向） | `-1` |
| `ss_runtime_get_track_playback_style(ctx, track)` | `int32_t`（`0` = 通常 / `1` = 往復） | `-1` |
| `ss_runtime_set_track_loop_count(ctx, track, count)` | `bool` | `false`（トラックにアニメーションが無い） |
| `ss_runtime_get_track_loop_count(ctx, track)` | `int32_t`（`-1` = 無限） | `0` |
| `ss_runtime_is_track_finished(ctx, track)` | `bool` | `false` |
| `ss_runtime_track_just_looped(ctx, track)` | `bool` | `false` |
| `ss_runtime_set_track_outgoing_event_threshold(ctx, track, threshold)` | `bool` | `false` |
| `ss_runtime_get_track_outgoing_event_threshold(ctx, track)` | `float` | `NaN` |
| `ss_runtime_get_current_animation_index(ctx)` | `int32_t`（track 0 の主アニメーション） | `-1`（何もセットアップしていない） |

### 7-2. 切り替え

| 関数 | 戻り値 | 失敗したとき |
|---|---|---|
| `ss_runtime_crossfade(ctx, track, name, duration_ms)` | `bool` | `false` |
| `ss_runtime_crossfade_by_hash(ctx, track, hash, duration_ms)` | `bool` | `false` |
| `ss_runtime_fade_track(ctx, track, target_alpha, duration_ms)` | `bool` | `false` |
| `ss_runtime_get_track_fade_progress(ctx, track)` | `float`（0〜1） | `NaN`（フェード中でない） |
| `ss_runtime_cancel_track_fade(ctx, track)` | `bool` | `false` |

### 7-3. 1 つのトラックに複数のソース

| 関数 | 戻り値 | 失敗したとき |
|---|---|---|
| `ss_runtime_get_track_main_source(ctx, track)` | `uint32_t` のハンドル | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_add_track_source(ctx, track, name, weight)` | `uint32_t` のハンドル | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_add_track_source_by_hash(ctx, track, hash, weight)` | `uint32_t` のハンドル | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_set_source_weight(ctx, handle, weight)` | `bool` | `false` |
| `ss_runtime_get_source_weight(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_frame(ctx, handle, frame)` | `bool` | `false` |
| `ss_runtime_get_source_frame(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_get_source_animation_index(ctx, handle)` | `int32_t` | `-1` |
| `ss_runtime_set_source_event_threshold(ctx, handle, threshold)` | `bool` | `false` |
| `ss_runtime_get_source_event_threshold(ctx, handle)` | `float` | `NaN` |

## 8. WebAssembly

WebAssembly のバインディングは C-API と 1 対 1 です。失敗は例外で伝わり、値の無い `float` は `undefined` になります。

```typescript
ctx.setup_animation("idle");

ctx.set_track_animation(1, "look_around");
ctx.set_track_blend(1, 1);          // 1 = Additive
ctx.set_track_alpha(1, 0.4);

ctx.set_track_sync(0, 1);           // 1 = Start
ctx.crossfade(0, "run", 300.0);
const progress = ctx.get_track_fade_progress(0);   // number | undefined

const run = ctx.add_track_source(0, "run", 0.3);
ctx.set_source_weight(run, 0.5);

ctx.set_track_animation(2, "attack");
ctx.set_track_loop_count(2, 1);     // 1 回再生して、最後のフレームで止まる
const done = ctx.is_track_finished(2);
```

## 9. 制約

- **離散的なアトリビュートは track 0 の主アニメーションから来ます。** track 0 のクロスフェードが始まると、入ってくる側のものに替わります。セル・パーツカラー・シェーダ・反転・非表示・描画優先度・テクスチャは混ぜられないので、重ねたトラックからセルを差し替えることはできません。セルを差し替えるには [パーツオーバーライド](./20_part_overrides.ja.md) を使ってください。
- **積む全アニメーションで、同じパーツに同じセルを割り当ててください。** セルの大きさと原点は track 0 から取るので、セルが違うと大きさや原点がずれて姿勢が崩れます。同じ ssae の中のアニメーションなら、ふつうは問題になりません。
- **パーツカラーは track 0 に従います。** パーツカラーを持つパーツでは、その色とアルファが優先されます。混ぜた不透明度が頂点の色に届くのは、パーツカラーを持たないパーツだけです。同じキャラクターのアニメーションどうしを混ぜるふつうの用途では、問題になりません。
- **位相はそろいません。** 尺の違うソースは、それぞれの時計で進みます。関係のない周期（呼吸とまばたき）ではそれが正しい挙動です。30 フレームの歩きと 60 フレームの走りをそろえたい場合は、Player がシークしてください。
- **パーツ単位のマスクはありません。** トラックが効く範囲は、そのアニメーションがどこにキーを打っているかで決まります（§1-2）。
- **ssab は 1 つです。** 別の ssab のアニメーションは積めません。

## 10. 性能

- **アニメーションを 1 本だけ再生するときの追加のコストはありません。** `alpha` が 1 のトラック 1 つにアニメーション 1 本、という状態では、合成はポインタの入れ替えだけになります。
- **コストはソースの数ではなく、パーツの数に比例します。** 有効なソースはそれぞれ 1 回計算され、重ねる処理はパーツの数に比例します。
- **重みが 0 のソースは計算しません**（フェードをキャンセルした後に残ったものを含む）。
- 複数のソースを混ぜているときは、描画に使わないソースの頂点変形のデータも FrameData に入るので、FrameData が大きくなります。結果には影響しません。

---
[目次に戻る](../README.ja.md)
