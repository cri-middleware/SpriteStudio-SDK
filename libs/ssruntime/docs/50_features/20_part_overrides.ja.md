> **日本語** | [English](./20_part_overrides.md)

# パーツオーバーライド

実行時に、**特定のパーツのカラー・セル・表示**を、アニメーションのキーフレームより優先して上書きする機能です。「このパーツを今この色にする」「この装備のセルに差し替える」「この部位だけ隠す」といった操作に使います。オーバーライドの状態は Brain が持つので、Player は一度設定すれば、毎フレーム設定し直す必要はありません。Player の公開 API での引数の順と既定値は [Player API 規約 §1](../20_design/30_api_conventions.ja.md#1-引数の順) と [§6](../20_design/30_api_conventions.ja.md#6-既定値) にあります。

## 1. 考え方

### 1-1. キーフレームより優先される理由

オーバーライドは、1 フレームの計算の **決まった 1 か所** で適用されます。

```
ソースごとにキーフレームを計算する
        ↓
   重みを付けて混ぜる（アニメーションミキシング）
        ↓
 ★ オーバーライドで上書きする（ここ） ★
        ↓
   FrameData を組み立てる
```

混ぜた**後**、出力の**前**に、決まった姿勢を上書きします。そのため、オーバーライドは**キーフレームの値もミキシングの結果も上書きします。** オーバーライドは置き換えで、ミキシング（重みを付けた合成）とは別の段階です（[アニメーションミキシング](./10_animation_mixing.ja.md)）。

### 1-2. 上書きできるもの

| 機能 | 効くパーツ | 優先度モード |
|---|---|---|
| **パーツカラー** | 通常のパーツだけ | あり |
| **セル** | 通常のパーツとマスクパーツ | あり |
| **表示（強制的に隠す）** | すべてのパーツ（子孫にも波及させられる） | **なし** |

**効かないパーツに設定しても、`set` は成功して `true` を返します。** たとえばヌルパーツにカラーのオーバーライドを設定しても、適用するときに無視されるだけです。呼び出しの成否で、効いたかどうかを判断しないでください。

次のものはオーバーライドの対象ではありません。

| したいこと | 実装する場所 |
|---|---|
| テクスチャ・セルマップの差し替え | **Player**。`DrawBatch.texture_hash` をキーにバインドするテクスチャを差し替える（[外部リソースとテクスチャ §3](./30_external_resources.ja.md#3-実行時のテクスチャの差し替え)）。作画で指定されたテクスチャ変更（`TCHG`）は Brain が計算済みで、`texture_hash` に反映されている |
| 反転 | **Player**。描画の変換で反転する |
| オブジェクト全体の色・不透明度・描画順 | **Player**。パーツ単位のことではない |
| インスタンスパーツの参照先の差し替え | できない |

### 1-3. 優先度モード

パーツカラーとセルは、アニメーションが同じアトリビュートにキーを打っていると競合します。そこで、**キーフレームとオーバーライドのどちらが勝つか**を 3 段階で指定します。表示は競合しないので、優先度モードを持ちません（常にオーバーライドが勝ちます）。

| 値 | 名前 | 意味 | 別のアニメーションをセットアップしたとき |
|---|---|---|---|
| `0` | `OverwriteOnNextKeyframe` | **キーフレームに譲る。** そのアトリビュートのキーフレームが来たら解除される | 消える |
| `1` | `HoldUntilNextAnimation` | **いまのアニメーションの間はオーバーライドが勝つ**（ふつうはこれ） | 消える |
| `2` | `Permanent` | オーバーライドが勝ち続け、**アニメーションを切り替えても残る** | 残る（パーツの数で確かめ直す） |

`0` は「次のキーフレームで取り返される」一時的な上書き、`1` は「このアニメーションの間の設定」、`2` は「この ssab を再生している間の設定」と考えてください。「アニメーション」は `ss_runtime_setup_animation` から次の `ss_runtime_setup_animation` までのことです。消えるきっかけは §4 にあります。

「キーフレームが来た」とは、トラック 0 が再生しているアニメーションが、そのパーツのそのアトリビュート（パーツカラーかセル）にキーを持つフレームを、再生が通過したことです。`ss_runtime_update` で通過したフレームを、次のフレームを組み立てるときに調べます。数えるのは再生ヘッドが実際に通ったフレームだけなので、再生区間・ループ・逆再生・往復再生でも特別な扱いは要りません。シークは、着地したフレームを数えます。オーバーライドを設定したフレームは、設定後に最初にフレームを組み立てるときと、そこから次の `ss_runtime_update` までにフレームを組み立てるときには数えません。そのため、キーフレームの上で設定すると、フレームを何度組み立てても、そのキーフレームの区間はオーバーライドが勝ちます。`play` やシークの後の `ss_runtime_update` は開始フレームを通過するので、そこで設定したオーバーライドがその区間で勝つのは、設定してからその update までにフレームを組み立てなかったときだけです。パーツの読み出しのためにフレームを組み立てる Player では、オーバーライドが外れます（[Player API 規約 §9](../20_design/30_api_conventions.ja.md#9-パーツの読み出し)）。`PartState.update_flag` は判定に使いません。

## 2. 使い方

```cpp
// 1. パーツ名からインデックスを引く（見つからなければ -1）
int32_t part = ss_runtime_find_part_index(ctx, "body");
if (part < 0) { /* パーツ名が違うか、アニメーションをセットアップしていない */ }

// 2. カラー: body に赤を乗算する（いまのアニメーションの間）
ss_runtime_set_part_color_override(ctx, (uint32_t)part,
                                   /*blend_op*/ 1,          // 1 = Mul
                                   /*rgba*/ 0xFF0000FF,     // 0xRRGGBBAA
                                   /*priority*/ 1);         // 1 = HoldUntilNextAnimation

// 3. セル: 別のセルを描かせる（名前はハッシュで渡す）
uint32_t cellmap = ss_runtime_hash_string("Ringo");        // .ssce は付けない
uint32_t cell    = ss_runtime_hash_string("effect3");
if (!ss_runtime_set_part_cell_override(ctx, (uint32_t)part, cellmap, cell, 1)) {
    // セルが見つからない。理由は ss_runtime_get_last_error
}

// 4. 表示: 強制的に隠し、子孫にも波及させる
ss_runtime_set_part_visibility_override(ctx, (uint32_t)part,
                                        /*force*/ 1,        // 1 = 強制的に隠す
                                        /*cascade*/ 1);     // 子孫にも波及させる

// 5. 解除する
ss_runtime_clear_part_color_override(ctx, (uint32_t)part);
ss_runtime_clear_all_part_overrides(ctx);                   // すべてのパーツ・すべての種類
```

C-API はパーツを **インデックス**（`part_index`）で指定します。パーツ名で操作する公開 API では、`ss_runtime_find_part_index` で引いたインデックスを Player がキャッシュしてください。インデックスは **同じ ssab の中で変わりません。**

複数のパーツをまとめて操作する C-API は、`ss_runtime_clear_all_part_overrides` だけです。「すべてのパーツ」を表す特別な値はありません。

## 3. それぞれのオーバーライド

### 3-1. カラーのオーバーライド

**通常のパーツだけ**に効きます。

- **合成の種類（`blend_op`）**: `0` = Mix / `1` = Mul / `2` = Add / `3` = Sub。キーフレームのパーツカラーと**同じ 4 種類**です（[パーツカラー §2](../40_parts/10_parts_color.ja.md#2-合成の種類)）。`4` 以上は `set` が `false` を返します（キーフレームのパーツカラーは Mix として扱いますが、こちらは**拒否**します）。
- **単色と 4 隅**: `ss_runtime_set_part_color_override` は 1 つの色を 4 隅すべてに使います。グラデーションにするには `ss_runtime_set_part_color_override_corners` を使います。**4 隅の順は `lt, rt, lb, rb`**（左上・右上・左下・右下）です。
- **`rate` は常に `1.0`** です。
- **アルファを二重に掛けないでください。** 指定した色のアルファには、キーフレームのパーツカラーと同じく、パーツの不透明度が掛けられます（[パーツカラー §3](../40_parts/10_parts_color.ja.md#3-パーツカラーを読む)）。
- **色は 8 bit の sRGB の値として扱われます。** Brain は色の計算をすべて 8 bit の sRGB の整数で行い、リニアやガンマや PMA の変換はしません。リニアへの変換が必要な Player は、キーフレームのパーツカラーと同じく、シェーダの入出力の関数（`ss_input_texture()` / `ss_output_color()`）でまとめて行ってください。オーバーライドのためだけに別の色の処理を作ると、変換が二重になります。

**キーフレームにパーツカラーを持たないパーツ**にカラーのオーバーライドを設定すると、そのパーツはパーツカラーを持つパーツとして出力されます（`update_flag` の `AttributePartColor` が立ち、FrameData にパーツカラーが載ります）。パーツカラーの有無で描き方を分けている Player は、この切り替わりを前提にしてください（[パーツカラー §5](../40_parts/10_parts_color.ja.md#5-パーツカラーが無いパーツ) のとおり描き方を分けていなければ、何もする必要はありません）。

### 3-2. セルのオーバーライド

**通常のパーツとマスクパーツ**に効きます。

セルは `(cellmap_name_hash, cell_name_hash)` で指定します。ハッシュは `ss_runtime_hash_string()` で作ります。セルマップ名には **`.ssce` を付けません。**

- **セルは `set` のときに 1 回だけ探します。** 見つからないセルマップ名やセル名は、**`set` の時点で `false`** になります（毎フレーム探すことはありません）。
- **`(0, 0)` を渡すと解除**され、アニメーションのセルに戻ります。

セルを差し替えると、次の 4 つが変わります。Player から見えるのは主に (2) と (3) です。

| # | 変わるもの | 影響 |
|---|---|---|
| (1) | 形（原点・大きさ・切り出し範囲） | 頂点に反映される |
| (2) | **`texture_hash`** | **DrawBatch の区切り方が変わる** |
| (3) | UV | `ss_runtime_get_local_uvs` などが差し替えたセルの UV を返す |
| (4) | セルの参照 | FrameData の `cell` が差し替えたセルを指す |

差し替えたセルが別のテクスチャなら、そのパーツは別の DrawBatch になります。Brain の描画計画に反映済みなので、Player は**いつもどおり `draw_batches` を読むだけ**で正しく描けます。「DrawBatch の数は一定」という前提の最適化はしないでください。

大きさ（`SizeX` / `SizeY`）のキーを持たないパーツでは、差し替えたセルの大きさがパーツの大きさになります（キーフレームでセルを変えたときと同じ）。

### 3-3. 表示のオーバーライド

- `force = 0` で解除、`force = 1` で強制的に隠します。`force = 2`（強制的に表示）は予約されていて、常に `false` を返します。
- **優先度モードはありません。** 常にキーフレームに勝ち、別のアニメーションをセットアップすると**必ず消えます**（`HoldUntilNextAnimation` と同じ動き）。
- `cascade != 0` で、子孫のすべてのパーツも隠します。

**子孫への波及は、オーバーライドで隠したときだけです。** SpriteStudio のキーフレームの非表示は**そのパーツだけ**のもので、子孫には及びません。この違いは意図したものです。Player が「親が非表示なら子も描かない」という波及を独自に作ると、キーフレームの非表示の意味が変わってしまいます。**`PartState.hide` は、そのパーツ自身を描くかどうかだけを表します。**

**インスタンスパーツの子アニメーションには届きません。** 子アニメーションは別の Context なので、親の Context のオーバーライドは子の中のパーツに効きません。インスタンスパーツ自体を隠したとき、子アニメーションを描かないのは Player の役目です。隠したインスタンスパーツの DrawBatch は出なくなるので、DrawBatch の位置で子を描いていれば（[インスタンスパーツ §5](../40_parts/50_instance.ja.md#5-子を描く)）、何もしなくても子は描かれません。

## 4. 寿命と `part_index`

オーバーライドの表は、`part_index` を添字とする配列で、Context が持ちます。`part_index` は **同じ ssab の中でだけ** 変わりません。別の ssab に替えたら、インデックスを引き直してください。

| きっかけ | 挙動 |
|---|---|
| `ss_runtime_setup_animation`（同じ ssab の別のアニメーション） | **表示のオーバーライドはすべて消える。** カラーとセルは `Permanent` **だけが残り**、`OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` は消える。残った `Permanent` は新しいパーツの数で確かめ直す（範囲外は捨てる） |
| track 0 への `ss_runtime_crossfade` / `ss_runtime_set_track_animation`（[主役](./10_animation_mixing.ja.md#1-1-track-0-は特別)が替わる） | **何も消えない。** 表示のオーバーライドも、`OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` も残る |
| `ss_runtime_bind_resource`（別の ssab をバインドする） | **すべて消える**（パーツが同じものでなくなるため） |
| `ss_runtime_reset` | すべて消える |

オーバーライドが 1 つも無いときは、上書きの処理がまるごと飛ばされます。この機能を使わない Player に、性能上の負担はありません。

## 5. 踏みやすい誤り

### 5-1. フレーム番号でキャッシュしていると効かない

オーバーライドは、**フレーム番号を変えずに出力を変えます。** 「前回と同じフレーム番号なら組み立てを省く」という最適化（一時停止中・止まったフレーム・低いフレームレートの再生で効く）を持つ Player では、`set` や `clear` をしても頂点が更新されず、**「API が効かない」ように見えます。** 動くものが無い tick を省く Player（[アニメーションミキシング §1-6](./10_animation_mixing.ja.md#1-6-動くものが無い-tick-を省く)）でも同じことが起きます。

フレームの組み立てを省く Player は、時計を進めずに次のフレームの見た目を変える呼び出しが成功するたびに、組み立て直しを要求してください。オーバーライドの `set` / `clear` のほか、トラック・ソース・フェード・クロスフェードの操作、シーク、再生区間の変更、アニメーションの切り替えが当たります。要求は 1 回きりにして、次にフレームを組み立てたときに取り下げてください。立てたままのフラグにすると、そのキャラクターの tick を二度と省けなくなります。

エンジンが毎フレーム書き込むことの多い Player 側の設定（キャラクターの置き方・表示・不透明度・色・サブフレームの切り替えなど）は、新しい値を前の値と比べ、変わったときだけ組み立て直しを要求してください。書き込むたびに要求すると、そのキャラクターの tick を省けなくなります。

### 5-2. `Direct` モードでは FrameData に `cells` が無い

`Direct` モード（既定）では、FrameData に `cells` / `vertices` / `deforms` を載せません（SoA バッファで渡すため）。`parts_color` は載ります。

そのため、**`FrameData.cells` を見ても、セルのオーバーライドが効いたかは分かりません**（「載っていない = 効いていない」と誤解しがちです）。`Direct` モードで確かめるには、次を見てください。

- `ss_runtime_get_local_uvs()` が返す UV（差し替えたセルの UV になる）
- `DrawBatch.texture_hash`（差し替えたセルが別のテクスチャなら変わる）

## 6. C-API の一覧

失敗したときは `false`（か `-1`）を返し、理由は `ss_runtime_get_last_error(ctx)` で取れます。`priority_mode` はどれも `0` = `OverwriteOnNextKeyframe` / `1` = `HoldUntilNextAnimation` / `2` = `Permanent`（§1-3）で、範囲外の値は `false` になります。

| 関数 | 引数 | 備考 |
|---|---|---|
| `ss_runtime_find_part_index(ctx, part_name)` | — | `int32_t`。見つからない、またはセットアップしていなければ `-1` |
| `ss_runtime_hash_string(str)` | — | `uint32_t`。`NULL` か UTF-8 でない文字列なら `0` |
| `ss_runtime_set_part_color_override(ctx, part, blend_op, rgba, priority_mode)` | `blend_op`: `0` = Mix `1` = Mul `2` = Add `3` = Sub<br>`rgba`: `0xRRGGBBAA` | 1 つの色を 4 隅に使う。通常のパーツだけ |
| `ss_runtime_set_part_color_override_corners(ctx, part, blend_op, rgba_ltrtlbrb, priority_mode)` | `rgba_ltrtlbrb`: `uint32_t` × 4（**`lt, rt, lb, rb` の順**） | 4 隅を別々に指定する |
| `ss_runtime_clear_part_color_override(ctx, part)` | — | |
| `ss_runtime_set_part_cell_override(ctx, part, cellmap_name_hash, cell_name_hash, priority_mode)` | セルマップ名とセル名のハッシュ | `(0, 0)` で解除。見つからなければ `false`。通常のパーツとマスクパーツだけ |
| `ss_runtime_clear_part_cell_override(ctx, part)` | — | |
| `ss_runtime_set_part_visibility_override(ctx, part, force, cascade)` | `force`: `0` = 解除 `1` = 強制的に隠す `2` = 予約<br>`cascade != 0` で子孫にも | 優先度モードなし |
| `ss_runtime_clear_part_visibility_override(ctx, part)` | — | |
| `ss_runtime_clear_all_part_overrides(ctx)` | — | すべてのパーツ・すべての種類を解除 |

---
[目次に戻る](../README.ja.md)
