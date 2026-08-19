> **日本語** | [English](./60_animation_mixing.md)

# アニメーションミキシング

複数のアニメーションを同時に再生します — 歩行に足を引きずる動きを重ねる、トラッキング値で首を振らせる、待機から攻撃へ遷移する。これらを **トラック** に積んで実現します。ミキシングはランタイム状態と FFI だけの機能で、`.ssab` の変更は不要です。

## 1. モデル

インスタンスはトラックのスタックを持ちます。トラックは番号の小さい順に、**セットアップポーズ**（`.ssae` が定義する素立ちのポーズ）の上へ畳み込まれます。

```
resolved = セットアップポーズ
track 0  →  重ねる   (メインのアニメーション)
track 1  →  重ねる   (オーバーレイ)
track 2  →  重ねる   (もう1枚)
```

つまみは2つあり、**どちらを使うかが重要**です。

| | 役割 | 単位 |
|---|---|---|
| **`alpha`** | そのトラックが下のポーズにどれだけ残るか。**強度のつまみはこちら** | トラックごと |
| **`weight`** | **同じトラック**に乗った複数アニメーション間の比率 | ソースごと |

トラックの `alpha` は常に期待どおりに効きます。一方ソースの `weight` は単体では期待どおりになりません — ある属性をキーしているアニメーションが1本だけなら、比率を取る相手がいないため、`weight` がどれだけ小さくてもその属性はそのアニメーションのものになります。**`weight` はアニメーション同士を本当に混ぜるとき（walk + run）だけに使い、それ以外は `alpha` を使ってください。**

### track 0 は特別

track 0 が**構造**を担当します。離散属性（セル・パーツカラー・シェーダ・反転・非表示・優先度・テクスチャ）とイベントの発火はすべて track 0 由来です。`setup_animation` はまさに「このアニメを track 0 に乗せる」という操作です。track 0 は clear できません — `alpha` をフェードしてください。

### トラックは「自分がキーしている属性」しか触らない

そのトラックのアニメーションがキーを持たない属性は、下のポーズがそのまま通り抜けます。頭だけをキーしたオーバーレイは、**マスクを指定しなくても**脚に影響しません。逆に言えば、**オーバーレイの効く範囲は作画した範囲そのもの**です。体の一部だけに効かせたいなら、その部分だけにキーを打ってください。

### 同一 SSAB のみ

1つの `AnimationResource`（1つの `.ssab`）内のアニメーション同士だけが積めます。

---

## 2. 単一アニメーションの再生

これまでどおりです。

```cpp
ss_runtime_setup_animation(context, "walk");   // == track 0
ss_runtime_update(context, delta_ms);
ss_runtime_get_frame_data(context, frame_no, &out_data, &out_len);
```

## 3. 重ねる

```cpp
ss_runtime_setup_animation(context, "idle");        // track 0: 本体の演技

ss_runtime_set_track_animation(context, 1, "look_around");
ss_runtime_set_track_alpha(context, 1, 0.4f);       // オーバーレイを 40%
```

トラックは必要に応じて生えます（track 3 を設定すると 1・2 も無音で作られます）。`alpha` はクランプしません — 1 を超えると外挿になり、オーバーレイではそれが欲しい場合があります。

### 加算トラック

`SS_RUNTIME_TRACK_BLEND_ADDITIVE` は、そのトラックのポーズへ寄せる代わりに、**セットアップポーズからの差分**を足します。下のアニメーションが残り続けます。

```cpp
ss_runtime_set_track_animation(context, 1, "lean_left");
ss_runtime_set_track_blend(context, 1, SS_RUNTIME_TRACK_BLEND_ADDITIVE);
ss_runtime_set_track_alpha(context, 1, 0.6f);       // 歩行の上に、傾きを 60%
```

連続値で制御する軸はこれを使います。軸は**中央フレームがセットアップポーズと一致する**ように作り、再生するかスクラブしてください。

```cpp
// 再生せず、トラッキング値でフレームを動かす
ss_runtime_set_track_frame(context, 1, 30.0f + yaw * 30.0f);
```

## 4. 遷移

`ss_runtime_crossfade` は、トラックが今再生しているものから新しいアニメーションへフェードします。ハンドルを組で管理する必要も、後始末も要りません。

```cpp
ss_runtime_crossfade(context, 0, "run", 300.0f);   // 300ms。update() の delta と同じ単位

ss_runtime_update(context, delta_ms);
float progress = ss_runtime_track_fade_progress(context, 0);  // 0..1、フェード中でなければ NaN
```

完了すると、入ってきたアニメーションがそのトラックのものになり、出ていったソースは解放されます。

| 状況 | 挙動 |
|---|---|
| `duration_ms <= 0` / `NaN` / 未知のアニメーション名 | `false`。詳細は `get_last_error` |
| 何も再生していないトラックへの `crossfade` | フェードせず即座に再生開始 |
| フェード中の同じトラックへ再度 `crossfade` | **実行中のフェードを先に完了させて**から、そこから新しいフェードを開始 |
| フェードを止めたい | `ss_runtime_cancel_track_fade`。alpha と weight はその時点の値で止まる |

### 素立ちへのフェードアウト

スタックの床はセットアップポーズなので、トラックをフェードアウトさせること自体が「素立ちへ戻す」ことになります。

```cpp
ss_runtime_fade_track(context, 0, 0.0f, 200.0f);   // 200ms かけてセットアップポーズへ
```

## 5. アニメーション同士を混ぜる

比率で混ぜたい場合（locomotion 合成）は、同じトラックに乗せて weight を設定します。

```cpp
ss_runtime_setup_animation(context, "walk");
uint32_t walk = ss_runtime_track_main_source(context, 0);
uint32_t run  = ss_runtime_track_add_source(context, 0, "run", 0.0f);

// walk 70% / run 30%
ss_runtime_set_source_weight(context, walk, 0.7f);
ss_runtime_set_source_weight(context, run, 0.3f);
ss_runtime_set_source_frame(context, run, 0.0f);
```

`ss_runtime_track_main_source` は、`set_track_animation`（や `setup_animation`）がそのトラックに載せたアニメーションのハンドルを返します。weight 1 から始まります。

weight は絶対値ではなく比率です（`(0.7, 0.3)` と `(7, 3)` は同じ結果）。各ソースは独立した再生クロックを持ち、`ss_runtime_update(delta)` はそれら全てを進めます。**尺の違うソースの位相は同期しません** — 揃えたい場合は自分でシークしてください。

## 6. イベント

既定では、そのトラックの完全な重みを持つアニメーションだけがイベントを発火します。ソースごとにしきい値を現在の weight より下げると発火に参加します。

```cpp
ss_runtime_set_source_event_threshold(context, run, 0.5f);  // run が 0.5 に達したら発火
```

イベントは発火したソースのハンドル付きで返るので、複数ソースを区別できます。

```cpp
int n = ss_runtime_get_passed_event_count(context);
for (int i = 0; i < n; ++i) {
    int32_t  event_idx = ss_runtime_get_passed_event_index(context, i);
    uint32_t src       = ss_runtime_get_passed_event_source_handle(context, i);
}
```

---

## 7. C-API リファレンス

### 7.1 トラック

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_set_track_animation(ctx, track, name)` | `bool` | `false` |
| `ss_runtime_set_track_animation_by_hash(ctx, track, hash)` | `bool` | `false` |
| `ss_runtime_set_track_alpha(ctx, track, alpha)` | `bool` | `false` |
| `ss_runtime_track_alpha(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_blend(ctx, track, blend)` | `bool` | `false` |
| `ss_runtime_track_blend(ctx, track)` | `uint32_t`（`0`=mix / `1`=additive） | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_track_count(ctx)` | `uint32_t` | `0` |
| `ss_runtime_clear_track(ctx, track)` | `bool` | `false`（track 0 は拒否） |
| `ss_runtime_set_track_frame(ctx, track, frame)` | `bool` | `false` |
| `ss_runtime_track_frame(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_speed(ctx, track, speed)` | `bool` | `false` |

### 7.2 遷移

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_crossfade(ctx, track, name, duration_ms)` | `bool` | `false` |
| `ss_runtime_crossfade_by_hash(ctx, track, hash, duration_ms)` | `bool` | `false` |
| `ss_runtime_fade_track(ctx, track, target_alpha, duration_ms)` | `bool` | `false` |
| `ss_runtime_track_fade_progress(ctx, track)` | `float`（0-1） | `NaN`（フェード中でない） |
| `ss_runtime_cancel_track_fade(ctx, track)` | `bool` | `false` |

### 7.3 1トラックに複数ソース

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_track_main_source(ctx, track)` | `uint32_t` ハンドル | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_track_add_source(ctx, track, name, weight)` | `uint32_t` ハンドル | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_track_add_source_by_hash(ctx, track, hash, weight)` | `uint32_t` ハンドル | 同上 |
| `ss_runtime_set_source_weight(ctx, handle, weight)` | `bool` | `false` |
| `ss_runtime_source_weight(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_frame(ctx, handle, frame)` | `bool` | `false` |
| `ss_runtime_source_frame(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_event_threshold(ctx, handle, threshold)` | `bool` | `false` |
| `ss_runtime_source_event_threshold(ctx, handle)` | `float` | `NaN` |

エラーコードは `ss_runtime_get_last_error(ctx)` で取得します。`NaN` を番兵にしているのは、正当な `0.0` と「そんなトラック/ソースは無い」を区別できるようにするためです。C では `isnan` / `isnanf`、JS では `Number.isNaN` で判定してください。

## 8. WASM (JS/TS)

Web バインディングは C-API と 1:1 です。エラーは例外として伝わり、`Option<f32>` は `undefined` になります。

```typescript
ctx.setup_animation("idle");

ctx.set_track_animation(1, "look_around");
ctx.set_track_blend(1, 1);          // 1 = additive
ctx.set_track_alpha(1, 0.4);

ctx.crossfade(0, "run", 300.0);
const progress = ctx.track_fade_progress(0);   // number | undefined

const run = ctx.track_add_source(0, "run", 0.3);
ctx.set_source_weight(run, 0.5);
```

---

## 9. 制限事項

- **離散属性は track 0 から取られます。** セル・パーツカラー・シェーダ・反転・非表示・優先度・テクスチャは合成されないため、オーバーレイのトラックからセルを差し替えることはできません。それには [Override Layer](./65_override_layer.ja.md) を使ってください。
- **`parts_color` は track 0 に従います。** `parts_color` を持つパーツでは、その色と α が支配的になります。合成された `alpha` が頂点カラーに届くのは `parts_color` を持たないパーツだけです。同一キャラクターのアニメーション同士を混ぜる一般的な用途では問題になりません。
- **位相同期はありません。** 尺の違うソースは独立したクロックで動きます。無関係な周期（呼吸と瞬き）ではそれが正しい挙動です。30 フレームの walk と 60 フレームの run を揃えたい場合は自分でシークしてください。
- **パーツ単位のマスクはまだありません。** トラックの効く範囲は、そのアニメーションが何をキーしているかで決まります（§1）。
- **`.ssab` は1つ。** 別々の `.ssab` のアニメーションは積めません。

## 10. パフォーマンス

- **単一アニメーション再生のコストはゼロです。** 完全な alpha のトラック1枚にアニメーション1本、という状態では、合成はポインタの swap に潰れます。
- **コストはソース数ではなくパーツ数に比例します。** アクティブなソースはそれぞれ1回評価され、畳み込み自体はパーツ数に比例します。
- **無音のソースはスキップされます。** weight 0 のソース（クロスフェード後に残ったものを含む）はコストになりません。

---

[目次に戻る](../README.ja.md)
