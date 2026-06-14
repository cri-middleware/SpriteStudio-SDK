# アニメーションブレンド / ミキシング (Phase 3)

複数のアニメーションを重み付きで合成し、滑らかな遷移 (state machine のクロスフェード) や locomotion ブレンド (walk + run の混合) を実現する API です。Phase 3 で追加された機能で、`.fbs` 変更を伴わず FFI と runtime state のみで構成されます。

## 1. 概念モデル

### Primary と Secondary
- **Primary**: アニメーションの「構造ソース」。Cell / blend_type / shader / 可視性などの**離散的**に決まる属性、および events の発火元はすべて primary から取られます。
- **Secondary**: TRSA / size / pivot / UV / α などの**連続値**の重み付き合成に参加。

### 連続値 vs 離散値の扱い

| 種類 | 取扱い | 例 |
|---|---|---|
| 連続値 | active な source の重み付き線形和 (auto-normalize) | position_xyz, rotation_xyz, scale, local_scale, α, local_α, pivot, anchor, size, uv_*, bounding_radius, mask, skew, `raw_vertex_deform` |
| 離散値 | **primary からコピー** | cell index, part_color, shader, deform table, flip_h/v, hide, img_flip, priority, texture, mesh_deform |
| `update_flag` | active source 間で OR 合成 | — |

`priority` は f32 ながら描画 z-order の sort key として用いるため、合成値の振動による Z-order チラつきを避ける目的で離散扱い (primary から取得) としています。

### Active source の条件
`animation_idx >= 0 && weight > 0.0` の slot のみが連続値合成に参加します。weight ≤ 0 は inactive (寄与なし)。

**Primary 例外**: Primary が `is_active() == false` (weight = 0) であっても、構造ソースとして必ず `extract_pose` が走ります。weight = 0 の primary は「Cell や離散属性は私が決めるが、連続値は secondary に任せる」という意味になります。

### Weight の自動正規化
weight は絶対値ではなく相対比率として解釈されます。`(0.7, 0.3)` と `(7.0, 3.0)` は同じ合成結果を生みます (内部で合計 1 に正規化)。

### 同一 SSAB 制約
ブレンドできるのは同じ `AnimationResource` (= 同一 SSAB) 内のアニメーション同士のみ。Player 側で複数 SSAB をまたぐ合成はサポートされません。

---

## 2. 基本的な使用法

```cpp
// 1. 通常通り Primary をセットアップ (この時点で blend_source_count = 1)
ss_runtime_setup_animation(context, "walk");

// 2. Secondary を追加。stable な handle が返る
uint32_t run = ss_runtime_add_blend_source(context, "run", 0.5f);
if (run == SS_RUNTIME_INVALID_HANDLE) {
    // setup_animation 未実行 / 名前不一致など。詳細は get_last_error。
}

// 3. Walk と Run を 50/50 ブレンド
ss_runtime_set_source_weight(context, /*primary*/ 0, 0.5f);
ss_runtime_set_source_weight(context, run, 0.5f);

// 4. 通常通り update / get_frame_data
ss_runtime_update(context, delta_ms);
uint8_t* out_data; size_t out_len;
ss_runtime_get_frame_data(context, frame_no, &out_data, &out_len);
```

---

## 3. Per-source の再生制御

各 source は独立した PlaybackController を持ち、独自の frame / speed を保持します。

```cpp
// Secondary の frame を明示的に seek
ss_runtime_set_source_frame(context, run, 0.0f);

// Secondary の再生速度を変更 (walk の半速で再生)
ss_runtime_set_source_speed(context, run, 0.5f);

// 現在の frame / weight を取得。handle 無効・inactive のとき NaN
float w = ss_runtime_source_weight(context, run);
if (!isnanf(w)) { /* 有効値 */ }
```

### `ss_runtime_update(delta)` の挙動
**全 active source** の controller を delta 量進めます。クロスフェード中の fade-in 側 source も自然に時間進行 → State machine 用途で `Context::update` を 1 回呼ぶだけで全 source が同期して進みます。

### `ss_runtime_get_frame_data(frame_no)` の挙動
- **Primary**: 引数の `frame_no` で評価 (single-animation 時の挙動を維持)。
- **Secondary**: 各 source の controller の現在 frame (`get_frame_no`) で評価。
- 引数 `frame_no` が受け付けるのは半開区間 `[start_frame, end_frame + 1)` で、終端区間 `(end_frame, end_frame + 1)` は最終キーフレームを保持して返します。詳細は [6. サブフレーム再生 — 受け付けるフレーム範囲と終端の扱い](./06_playback_control.ja.md#受け付けるフレーム範囲と終端の扱い) を参照。

---

## 4. State machine クロスフェードのパターン

### 4.1 Auto-crossfade ヘルパー (推奨)

`ss_runtime_crossfade(ctx, from, to, duration_ms)` で線形クロスフェードを runtime に任せられます。`update(delta)` 呼出毎に timer が進み、`(from.weight, to.weight)` が `(1, 0)` → `(0, 1)` に補間されます。完了時に `to` が primary に昇格し、`from` は自動的に deactive 化 (`animation_idx = -1`) されます。

```cpp
ss_runtime_setup_animation(context, "walk");
uint32_t walk = ss_runtime_primary_handle(context);
uint32_t run = ss_runtime_add_blend_source(context, "run", 0.0f);
ss_runtime_crossfade(context, walk, run, 300.0f);  // 300ms = update() の delta と同単位

// フェード進行は update() に任せて、進捗だけ問い合わせ可能
ss_runtime_update(context, delta_ms);
float progress = ss_runtime_crossfade_progress(context);  // 0.0-1.0、未稼働時 NaN
```

完了後の状態:
- `primary_handle()` == `run`
- `walk` slot は inactive (`source_weight(walk)` は NaN を返す)
- `crossfade_progress()` は NaN

`set_primary` の瞬間に構造ソース (Cell 等) が `to` へ swap されます。fade 末尾で fade-in 側 secondary の weight が 1.0 になる仕組みのため、視覚的不連続は最小化されています。

### 4.2 エラーと in-flight の扱い

| 状況 | 挙動 |
|---|---|
| `from == to` / `duration_ms <= 0` / `NaN` / 無効ハンドル | `crossfade` が `false` 返却、`last_error = InvalidParameter` |
| `from` が empty / inactive | 同上 (`from` は real animation 必須) |
| `to` が empty source | 受け付ける。完了時 primary は swap せず、`to` の weight = 1.0 で停止 (setup pose へ fade-out) |
| `crossfade` 中に `crossfade` 再呼出 | 旧 fade を破棄、新 fade を即開始。`from`/`to` の weight は再度 `(1, 0)` から始まる |
| `crossfade` 中に `remove_blend_source(from)` or `(to)` | fade を自動キャンセル |
| 手動で fade を止めたい | `ss_runtime_cancel_crossfade(ctx)`。weight は呼出時点の値を維持 (caller 側で正規化) |

### 4.3 旧来の手動制御 (互換)

`crossfade` を使わず毎フレーム自前で weight を駆動するパターンも引続きサポートされます。state machine のような複合 transition (curve, ease, hold-previous) を実装する場合はこちら:

```cpp
float t = elapsed / fade_duration;
ss_runtime_set_source_weight(context, walk, 1.0f - t);
ss_runtime_set_source_weight(context, run, t);
// 完了タイミングと slot 後始末は呼出側責務
ss_runtime_set_primary(context, run);
ss_runtime_remove_blend_source(context, walk);
```

---

## 4.4 Empty source (setup pose fade-out)

実 animation を持たない「identity pose source」を `ss_runtime_add_empty_blend_source(ctx, weight)` で追加できます。primary を setup pose 方向にフェードアウトしたいときに使います (アタック動作の終わりを自然に静止姿勢へ戻す、など)。

**Empty source の contributions** (mix 時):

| Attribute | Value |
|---|---|
| `scale_*`, `local_scale_*`, `alpha`, `local_alpha`, `uv_scale_*` | `1.0` |
| `position_*`, `rotation_*`, `pivot_*`, `anchor_*`, `uv_translation_*`, `uv_rotation_*`, `skew_*`, `mask`, `bounding_radius`, `size_*` | `0.0` |
| `update_flag` | empty (primary の flag が支配) |
| `raw_vertex_deform` | `present=false` (寄与なし) |

```cpp
ss_runtime_setup_animation(context, "attack");
uint32_t attack = ss_runtime_primary_handle(context);
uint32_t empty = ss_runtime_add_empty_blend_source(context, 0.0f);
ss_runtime_crossfade(context, attack, empty, 200.0f);  // attack pose を setup pose へ fade
```

完了時:
- primary は `attack` のまま (empty は primary に昇格しない)
- attack の weight = 0、empty の weight = 1
- resolved pose は identity pose

その後 caller は `ss_runtime_remove_blend_source(empty)` で empty を片付けるか、別 animation を `crossfade` で fade-in できます。

**制限**:
- Empty source に対する `set_source_frame` / `set_source_speed` / `set_source_event_threshold` / `set_primary` は `InvalidParameter` を返します (controller / animation を持たないため)。
- `set_source_weight` / `source_weight` / `remove_blend_source` / `clear_blend_sources` は他の source と同様に動作します。
- `add_empty_blend_source` は real primary がセットアップされていないと `AnimationNotFound`。Primary は常に real animation でなければなりません (離散属性 / events の権威ソース)。
- Empty は「identity pose への補間」であって「invisible への補間」ではありません。視覚的フェードアウト (alpha → 0) が欲しい場合は primary 側 keyframe / Player レイヤーの alpha で実現してください。

---

## 4.5 Per-source イベント発火 (Event threshold)

デフォルトでは Primary のみがイベントを発火します (`event_threshold = 1.0` で全 source 初期化)。Secondary もイベントを発火させたい場合、`ss_runtime_set_source_event_threshold(ctx, handle, threshold)` で source 毎の閾値を下げます。

**Firing rule**: `source.weight >= source.event_threshold` のとき発火 (Primary は閾値を無視して常に発火)。

```cpp
uint32_t walk = ss_runtime_primary_handle(context);
uint32_t run = ss_runtime_add_blend_source(context, "run", 0.0f);
ss_runtime_set_source_event_threshold(context, run, 0.5f);
// run の weight が 0.5 以上に達した瞬間からイベント発火開始
```

複数 source が同一 frame で発火するケースを区別するため、parallel array で source handle を取得します:

```cpp
int n = ss_runtime_get_passed_event_count(context);
for (int i = 0; i < n; ++i) {
    int32_t event_idx = ss_runtime_get_passed_event_index(context, i);
    uint32_t src     = ss_runtime_get_passed_event_source_handle(context, i);
    // event_idx は src が指す source の animation events table への index
}
```

`src == primary_handle()` の場合は legacy 挙動 (primary 発火)。`src` が secondary handle のときは、`src` で取得した source の animation メタデータを使ってイベント詳細を解決してください (Player 側で source → animation のマッピングを保持)。

ユースケース:
- クロスフェード後半で新アニメの足音イベントを発火させたい → `set_source_event_threshold(new_source, 0.5)`
- ダメージ overlay は引き続きイベント抑止 (default 1.0 のまま)

---

## 5. Handle と slot のライフサイクル

- `add_blend_source` は末尾の slot に append し、安定な `u32` ハンドルを返却。
- `remove_blend_source` は slot を **inactive にマーク** (`animation_idx = -1`)。slot 自体は配列に残るため、既存ハンドルは不変・他の handle のシフトなし。
- `clear_blend_sources` は primary 以外の全 slot を inactive。Primary は無傷。
- Primary slot は `remove_blend_source` できません (`InvalidParameter` 返却)。Primary を変えたい場合は先に `set_primary` で別 slot を primary 化。

---

## 6. C-API リファレンス

### 6.1 ソース管理

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_add_blend_source(ctx, name, weight)` | `uint32_t` handle | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_add_blend_source_by_hash(ctx, hash, weight)` | `uint32_t` handle | 同上 |
| `ss_runtime_add_empty_blend_source(ctx, weight)` | `uint32_t` handle (empty source) | 同上 |
| `ss_runtime_remove_blend_source(ctx, handle)` | `bool` | `false` |
| `ss_runtime_clear_blend_sources(ctx)` | void | — |
| `ss_runtime_blend_source_count(ctx)` | `uint32_t` (**primary + empty を含む** active 数) | `0` |
| `ss_runtime_set_primary(ctx, handle)` | `bool` | `false` (empty 拒否) |
| `ss_runtime_primary_handle(ctx)` | `uint32_t` | `SS_RUNTIME_INVALID_HANDLE` |

### 6.2 Per-source 再生制御 / weight

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_set_source_weight(ctx, handle, weight)` | `bool` | `false` |
| `ss_runtime_source_weight(ctx, handle)` | `float` | `NaN` (invalid/inactive) |
| `ss_runtime_set_source_frame(ctx, handle, frame)` | `bool` | `false` (empty 拒否) |
| `ss_runtime_source_frame(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_speed(ctx, handle, speed)` | `bool` | `false` (empty 拒否) |

### 6.3 Auto-crossfade

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_crossfade(ctx, from, to, duration_ms)` | `bool` | `false` |
| `ss_runtime_crossfade_progress(ctx)` | `float` (0.0-1.0) | `NaN` (未稼働) |
| `ss_runtime_cancel_crossfade(ctx)` | void | — |

### 6.4 Per-source events

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_set_source_event_threshold(ctx, handle, threshold)` | `bool` | `false` (empty 拒否) |
| `ss_runtime_source_event_threshold(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_get_passed_event_count(ctx)` | `int32_t` | — |
| `ss_runtime_get_passed_event_index(ctx, i)` | `int32_t` (animation 内 event index) | `-1` |
| `ss_runtime_get_passed_event_source_handle(ctx, i)` | `uint32_t` (発火元 source handle) | `SS_RUNTIME_INVALID_HANDLE` |

エラーの詳細コードは `ss_runtime_get_last_error(ctx)` で取得。`NaN` 番兵を採用しているのは「`weight = 0.0` (有効値)」と「無効ハンドル」を呼び出し側が区別できるようにするためです。C では `isnan` / `isnanf`、JS では `Number.isNaN` で判定できます。

---

## 7. WASM (JS/TS) バインディング

Web bindings は C-API を 1:1 ミラー。`Result<(), JsValue>` でエラーを伝搬し、`Option<f32>` は JS 側で `undefined` を返します。

```typescript
const ctx = /* SSRuntimeContext */;
ctx.setup_animation("walk");
const run = ctx.add_blend_source("run", 0.5);
ctx.set_source_weight(run, 0.3);

const w = ctx.source_weight(run);  // number | undefined
if (w !== undefined) {
    // ...
}

// Auto-crossfade
ctx.crossfade(walk, run, 300.0);
const progress = ctx.crossfade_progress();  // number | undefined

// Empty source for setup-pose fade-out
const empty = ctx.add_empty_blend_source(0.0);
ctx.crossfade(walk, empty, 200.0);

// Per-source event threshold + parallel handle
ctx.set_source_event_threshold(run, 0.5);
const src_handle = ctx.get_passed_event_source_handle(0);  // u32 (0xFFFFFFFF if out-of-range)
```

---

## 8. 既知の制限

### parts_color の α premultiplication
`extract_pose` (Stage A) 内で各 source の `parts_color` は自身の hierarchy α と pre-multiply されます。Mix 後の `vertex color` は primary の `parts_color_vec` を verbatim 使用するため、**primary の α が embedded** された状態となり、`state.alpha` (mixed) とは別系統の値になります。

**実用上の挙動**:
- `parts_color` を持たない Part: mixed `state.alpha` がそのまま視覚 α になり、滑らかにブレンド。
- `parts_color` を持つ Part: primary の色と α が支配的 (secondary の `parts_color` は使われない)。

同一キャラクタの locomotion blending (= 一般的なユースケース) では問題になりません。色そのものをブレンドしたい場合は将来 PR で「primary の α を un-premultiply → mixed α で re-premultiply」する補正を検討します。

### Cell の一貫性
ブレンドする全アニメーションの「同一 part に同一 Cell が割り当てられている」前提です。`cell_meta_per_part` (size, pivot, texture_hash 等) は primary から取得されるため、異なる Cell が割り当てられている場合、secondary の TRSA mix と整合しない pose になる可能性があります。

同一 SSAE 内の異なるアニメーションは通常同じ Cell 割当を共有するため、典型的なユースケースでは問題になりません。

### deform table の FB バッファ浪費
Stage A の per-source ループで secondary source の deform table も FlatBuffer に書き込まれますが、`resolved.deform_vec` は primary のものを使うため、FrameData binary 内で unreferenced な dead data として残ります。FrameData サイズは膨らみますが正確性には影響しません。将来 PR で deform table の FB 書き込みを post-mix に遅延する最適化を検討します。

---

## 9. パフォーマンス特性

### N=1 fast path
**Primary のみが active source** のケース (= ブレンドを使わない単一アニメーション再生) では、mix 段が `mem::swap` に分岐し O(1) で完了します。ブレンド API を使わない Player にとっての性能 regression はありません。

### Multi-source mix
N source ブレンド時、N 回の `extract_pose` と 1 回の `mix_poses`。`mix_poses` のコストは N に依存せず一定 (per-part の加算ループのみ)。

### Inactive slot のスキップ
weight = 0 または `remove_blend_source` 済の secondary は Stage A をスキップ。State machine の「fade-out 完了後の slot を残す」運用で extract コストは発生しません。

---

[目次に戻る](./README.ja.md)
