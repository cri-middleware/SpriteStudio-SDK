> **日本語** | [English](./65_override_layer.md)

# パーツオーバーライド (Override Layer API)

実行時に、**特定のパーツのカラー / セル / 表示** を、アニメーションのキーフレームより優先して上書きする API です。「このパーツを今この色にする」「この装備セルに差し替える」「この部位だけ隠す」といった操作を担当します。

---

## 1. 概念モデル

### 1-1. キーフレームより優先される理由

オーバーライドは、1 フレームの評価パイプラインの**特定の 1 点 (seam)**で適用されます。

```
source ごとのキーフレーム展開
        ↓
   重み付きミックス (ブレンド) 
        ↓
 ★ オーバーライド上書き (ここ) ★
        ↓
   compose / FrameData 出力
```
> 重み付きミックスの詳細は、[アニメーションミキシング](./60_animation_mixing.ja.md) を参照してください。

ミックスの**後**、出力の**前**に、解決済みポーズ (resolved pose) を上書きします。したがってオーバーライドは、**キーフレームの値やブレンド結果を上書きする**結果になります。

オーバーライドは上書き (置き換え) の機能であり、ブレンド(重み付き合成)などと混ぜて実装しないようにしてください。同じパイプラインであるものの、別のステージであることに留意してください。

> **Player は上書きを毎フレーム再適用しない**: オーバーライドの状態は Brain 側が保持します。1 度 `set` すれば、以降のフレームでは自動的に適用され続けます。「アニメーション更新のたびに再設定する」実装にしないでください。優先度モードは [§1-3](#1-3-オーバーライドの優先度モード) 、ライフサイクルは [§4](#4-ライフサイクルと-part_index) を参照してください。

### 1-2. 上書きの対象について

オーバーライドの対象となるアトリビュートは以下の通りです。

| 機能 | 適用対象パーツ | 優先度モード |
|---|---|---|
| **パーツカラー** | Normal のみ | あり |
| **セル** | Normal ＋ Mask | あり |
| **表示 (強制非表示)** | 全パーツ (子孫への cascade 可) | **なし** |

また、オーバーライド対象**外**のものと、その理由は次のとおりです。混同して「オーバーライドで実装しよう」としないでください。

| やりたいこと | 正しい実装場所（責務側） |
|---|---|
| テクスチャ / セルマップ (アトラス) の差し替え | **Player 側**。`DrawBatch.texture_hash` をキーにバインドするテクスチャを差し替える → [外部リソースとテクスチャの管理](./70_external_resources.ja.md)。なお**作画側**で指定されたテクスチャ変更 (`TCHG`) はランタイムが評価済みで、`texture_hash` に反映されています |
| 反転 (flip) | **Player 側**。FrameData の後処理 (スケール反転 / UV スワップ) で表現 |
| オブジェクト全体の色 / 不透明度 / 描画順 | **Player 側**。パーツ単位の関心事ではない |
| インスタンスパーツの差し替え | 将来別のステージで対応予定（**Phase 3** の Instance Lifecycle API） |

> **パーツ種別のガードは「適用時」に効きます**: 例えば Null パーツにカラーオーバーライドを設定しても、`set` 呼び出し自体は**成功して `true` を返します**が、seam で無視されるため何も起こりません。呼び出しの成否で「効いたかどうか」を判定しないでください。

### 1-3. オーバーライドの優先度モード

パーツカラーとセルは、アニメーションが同じアトリビュートを使用している場合に競合します。そこで、**キーフレームとオーバーライドのどちらが勝つか**を 3 段階で指定します。表示 (強制非表示) は競合しないため、優先度モードを持ちません (常に勝ちます)。

| 値 | 名称 | 意味 | 新しいアニメーションを `setup` したとき |
|---|---|---|---|
| `0` | `OverwriteOnNextKeyframe` | **キーフレームに譲る**。アニメーションの当該アトリビュートが更新されれば、オーバーライドは解除されます。 | クリアされる |
| `1` | `HoldUntilNextAnimation` | **現在のアニメーションの間は、オーバーライドが勝ち続ける** (標準的な選択) | クリアされる |
| `2` | `Permanent` | オーバーライドが勝ち続け、**アニメーション変更をまたいで生き残る** | 維持される (パーツ数で再検証) |

`0` は「キーフレームが次に来たら奪い返される」一時的な上書き、`1` は「このアニメーションの間の設定」、`2` は「`ssab` （アニメーションパック）を再生している間の恒久的な設定」と考えてください。

判定は `PartState` の `UpdateAttributeFlags` を見て行われます (`AttributePartColor` / `AttributeCell` のビットが立っていれば「そのフレームで新しいアトリビュートが更新された」となります)。

---

## 2. 基本的な使用法

```cpp
// 1. パーツ名 → part_index を解決 (見つからなければ -1)
int32_t part = ss_runtime_find_part_index(context, "body");
if (part < 0) { /* パーツ名が違う / setup_animation 未実行 */ }

// 2. カラー: body を赤で乗算 (現在のアニメーションの間は保持)
ss_runtime_set_part_color_override(context, (uint32_t)part,
                                   /*blend_op*/ 1,          // 1 = Mul
                                   /*rgba*/ 0xFF0000FF,     // 0xRRGGBBAA
                                   /*priority*/ 1);         // 1 = HoldUntilNextAnimation

// 3. セル: 別のセルを描画させる (名前はハッシュで渡す)
uint32_t cellmap = ss_runtime_hash_string("Ringo");    // .ssce 拡張子は付けない
uint32_t cell    = ss_runtime_hash_string("effect3");
if (!ss_runtime_set_part_cell_override(context, (uint32_t)part, cellmap, cell, 1)) {
    // セル名が解決できない → set 時点で即 false (詳細は get_last_error)
}

// 4. 表示: 強制非表示 + 子孫にも波及
ss_runtime_set_part_visibility_override(context, (uint32_t)part,
                                        /*force*/ 1,        // 1 = 強制非表示
                                        /*cascade*/ 1);     // 子孫にも波及

// 5. 解除
ss_runtime_clear_part_color_override(context, (uint32_t)part);
ss_runtime_clear_all_part_overrides(context);   // 全パーツ・全属性を一括解除
```

パーツの指定は **`part_index` (`uint32`) のみ**です。名前で操作したい場合は、`ss_runtime_find_part_index` で 1 度解決したインデックスを Player 側でキャッシュしてください。`part_index` は**同一 SSAB 内で安定**です。

> **「全パーツ」を表すセンチネル値はありません**: Brain 側のオーバーライド機能でパーツを横断する一括操作を行うのは `ss_runtime_clear_all_part_overrides` のみで、それ以外の一括操作は Player 側の責務となります。

---

## 3. 各オーバーライドの詳細

### 3-1. パーツカラーのオーバーライド

**Normal パーツのみ**に適用されます。

- **合成モード (`blend_op`)**: `0=Mix` / `1=Mul` / `2=Add` / `3=Sub`。キーフレームの PartColor と**同じ 4 種**です。合成式そのものは [パーツカラー §1-4](./50_parts_color.ja.md#1-4参考シェーダでのパーツカラーの適用例テーブル駆動の参考実装) を参照してください。`4` 以上は `set` 時点で `false` (パーツカラー側の「防御的に Mix へクランプ」とは異なり、こちらは**拒否**します)。
- **単色版と 4 隅版**: `ss_runtime_set_part_color_override` は 1 つの RGBA を 4 隅すべてに流します。グラデーションを与えたい場合は `ss_runtime_set_part_color_override_corners` を使います。
- **`rate` は常に `1.0`**: オーバーライドの合成強度は最大固定です。

> **4 隅の順序は `lt, rt, lb, rb`** (左上・右上・左下・右下) です。ここを取り違えると、グラデーションの方向が狂った状態になります。

> **α は二重に適用しないでください**: 指定した RGBA の `a` は、seam でそのパーツの**合成済み α と自動的に pre-multiply されます**。これはキーフレームの PartColor とまったく同じ規則です ([パーツカラー §1-3](./50_parts_color.ja.md#1-3参考パーツカラーに含まれるデータについて))。Player 側で `state->alpha()` を再乗算しないでください。

> **色は 8bit sRGB 空間で解釈されます**: Brain は色演算をすべて 8bit sRGB 整数空間で行い、linear / gamma / PMA 変換は一切行いません。オーバーライドの色も**キーフレームの PartColor とまったく同じ空間**として扱われます。linear 変換が必要な Player は、従来どおりシェーダ側のフック (`ss_input_texture()` / `ss_output_color()`) に集約してください。**オーバーライドのためだけに別の色パイプラインを作らないでください** (二重変換になります)。

**キーフレームに PartColor を持たなかったパーツ**にカラーオーバーライドを設定すると、seam で `AttributePartColor` フラグが立ちます。その結果、そのパーツは **5 頂点 (4 隅＋中心) 形式**で emit されるようになります ([パーツカラー §1-2-1](./50_parts_color.ja.md#1-2-1通常normalパーツの例) のセンター頂点)。「PartColor の有無で描画経路を分けている」Player は、この切り替わりが起こることを前提にしてください ([パーツカラー §1-2-4](./50_parts_color.ja.md#1-2-4パーツカラー無しcolor_idx--0の既定値) の推奨どおり経路を統一していれば、何もする必要はありません)。

### 3-2. セルのオーバーライド

**Normal パーツと Mask パーツ**に適用されます。

セルは `(cellmap_name_hash, cell_name_hash)` で指定します。セルマップ名に **`.ssce` 拡張子は含めません**。セルマップ名とセル名のハッシュは `ss_runtime_hash_string()` で生成してください。

- **解決は `set` 時に 1 回だけ**行われ、結果 (`ResolvedCellMeta`) がキャッシュされます。したがって**未知のセルマップ名 / セル名は `set` の時点で即 `false`** になります (毎フレームのハッシュ探索は発生しません)。
- **`(0, 0)` を渡すと解除**され、アニメーションのセルに戻ります。

セルを差し替えると、seam は次の **4 系統**を書き換えます。Player 側から見えるのは主に (2) と (3) です。

| # | 系統 | 影響 |
|---|---|---|
| (1) | ジオメトリ (pivot / size / rect) | 頂点計算に反映 |
| (2) | **`texture_hash`** | **`DrawBatch` の分割点が変わる** |
| (3) | UV | `ss_runtime_get_local_uvs` などが差し替え後のセルの UV を返す |
| (4) | セル参照 | FrameData の `cell` インデックスが差し替え後を指す |

> **差し替え先セルが別テクスチャなら、バッチ分割が変わります**: `texture_hash` が変われば、そのパーツは別の `DrawBatch` に切り出されます。これは Brain 側の描画プランニングに織り込み済みなので、Player は**通常どおり `draw_batches` を読むだけ**で正しく描画できます。ただし Player 側で、「バッチ数が固定である」ことを前提にした最適化を行っている場合は破綻しますので、注意してください。

**SizeX / SizeY のキーを持たないパーツ**は、差し替え後のセルのサイズが既定サイズとして再導出されます (キーフレーム側の「セルサイズへのフォールバック」と同じ挙動です)。

### 3-3. 表示状態のオーバーライド (強制非表示)

- `force = 0` で解除、`force = 1` で強制非表示。**`force = 2` (強制表示) は予約されていますが、現在は常に `false` を返します。**
- **優先度モードはありません。** 常にキーフレームに勝ち、新しいアニメーションを `setup` すると**必ずクリアされます**（動作として、優先度モード `HoldUntilNextAnimation` と等価です）。
- `cascade != 0` で、子孫パーツすべてに強制非表示が cascade （波及）します。

> **cascade は「オーバーライド由来の非表示」だけを伝播します**: SS7 のキーフレームの `hide` は**パーツ単位**であり、子孫に波及**しません**。この非対称性は意図的なものです。Player 側で「親が hide なら子も描かない」という独自の伝播を実装してしまうと、キーフレームの `hide` の意味が変わってしまいます。**`PartState.hide` は、そのパーツ自身を描くかどうかだけを表します。**

> **インスタンスパーツ配下には届きません**: 子アニメーションは別 context として動くため、親 context のオーバーライドは子の中のパーツには影響しません。インスタンスパーツ自体を強制非表示にしたとき、その配下の子アニメーションを描画しないのは **Player 側の責務**です。

---

## 4. ライフサイクルと part_index

オーバーライドのテーブルは `part_index` を添字とする密な配列で、context (インスタンス) が保持します。`part_index` は**同一 SSAB 内でのみ安定**です。別の `.ssab` に差し替えたら、インデックスは取り直してください。

| 契機 | 挙動 |
|---|---|
| `ss_runtime_setup_animation` (同一 SSAB 内の別アニメーション) | **表示オーバーライドは全消去**。カラー / セルは `Permanent` **のみ残り**、`OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` は消去。残った `Permanent` は新しいパーツ数で再検証 (範囲外は破棄) |
| `ss_runtime_bind_resource` (別の `.ssab` をバインド) | **全消去** (パーツの同一性が失われるため) |
| `reset` | 全消去 |

> **オーバーライドが 1 つも無いとき、seam の処理はまるごとスキップされます**: 「アクティブなオーバーライドが 1 つでもあるか」の判定で seam 全体がゲートされているため、この API を使わない Player にとって実質的な性能上のオーバーヘッドはありません。

---

## 5. Player 実装で踏みやすい罠

移植作業で実際に踏まれたものを挙げます。

### 5-1. フレーム番号ベースのキャッシュが効いてしまう

オーバーライドは **`frame_no` を変えずに出力を変えます**。「前回と同じフレーム番号なら再構築をスキップする」という最適化 (一時停止中・静止フレーム・低フレームレート再生で有効) を持つ Player では、`set` / `clear` してもジオメトリが更新されず、**「API が効かない」ように見えます**。

**対策**: オーバーライドの `set` / `clear` が成功したら、フレームキャッシュを明示的に無効化してください。これは Unity Player が実際に踏んだ罠です。

### 5-2. `RenderMode::Direct` では FrameData に `cells` が載らない

`RenderMode::Direct` (既定) では、`compose_frame_data` は **`cells` / `vertices` / `deforms` のベクタを FrameData に載せません** (これらは SoA バッファ経由で渡すため)。**`parts_color` は Direct でも載ります。**

したがって、**`FrameData.cells` を見てセルのオーバーライドの効果を検証することはできません** (「載っていない ＝ 効いていない」と誤診しがちです)。Direct モードでセル差し替えの結果を確認するには、次を見てください。

- `ss_runtime_get_local_uvs()` が返す UV (差し替え後のセルの UV になる)
- `DrawBatch.texture_hash` (差し替え先が別テクスチャなら変わる)

---

## 6. C-API リファレンス

すべて失敗時は `false` (または `-1`) を返し、詳細は `ss_runtime_get_last_error(ctx)` で取得できます。

### 6-1. パーツの解決

| 関数 | 戻り値 | エラー時 |
|---|---|---|
| `ss_runtime_find_part_index(ctx, part_name)` | `int32_t` (`part_index`) | `-1` (名前不一致 / `setup_animation` 未実行) |
| `ss_runtime_hash_string(str)` | `uint32_t` (ハッシュ値) | `0` |

### 6-2. カラー

| 関数 | 引数 | 備考 |
|---|---|---|
| `ss_runtime_set_part_color_override(ctx, part, blend_op, rgba, priority_mode)` | `blend_op`: `0=Mix 1=Mul 2=Add 3=Sub`<br>`rgba`: `0xRRGGBBAA` | 単色を 4 隅に流す。Normal パーツのみ |
| `ss_runtime_set_part_color_override_corners(ctx, part, blend_op, rgba_ltrtlbrb, priority_mode)` | `rgba_ltrtlbrb`: `uint32` × 4 (**`lt, rt, lb, rb` 順**) | 4 隅個別指定 |
| `ss_runtime_clear_part_color_override(ctx, part)` | — | |

### 6-3. セル

| 関数 | 引数 | 備考 |
|---|---|---|
| `ss_runtime_set_part_cell_override(ctx, part, cellmap_name_hash, cell_name_hash, priority_mode)` | cellmap_name_hash:セルマップ名のハッシュ<br>cell_name_hash:セル名のハッシュ | `(0, 0)` で解除。解決失敗で `false`。Normal ＋ Mask のみ |
| `ss_runtime_clear_part_cell_override(ctx, part)` | — | |

### 6-4. 表示

| 関数 | 引数 | 備考 |
|---|---|---|
| `ss_runtime_set_part_visibility_override(ctx, part, force, cascade)` | `force`: `0=解除 1=強制非表示 2=強制表示(未実装)`<br>`cascade != 0` で子孫へ波及 | 優先度モードなし |
| `ss_runtime_clear_part_visibility_override(ctx, part)` | — | |

### 6-5. 一括

| 関数 | 備考 |
|---|---|
| `ss_runtime_clear_all_part_overrides(ctx)` | 全パーツ・全属性を解除 |

`priority_mode` は全体で共通に `0=OverwriteOnNextKeyframe` / `1=HoldUntilNextAnimation` / `2=Permanent` です ([§1-3](#1-3-オーバーライドの優先度モード))。範囲外の値は `false`。

---

[目次に戻る](../README.ja.md)
