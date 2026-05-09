# バッチ描画と cross-engine 移植契約 (Batched Rendering & Cross-Engine Contract)

> **【ステータス】** このドキュメントは **forward-looking な契約仕様** です。記述されているスキーマ追加や FFI 動作は、コード実装に先んじて合意するための「契約書」として扱ってください。実装の進捗状況は [04_roadmap.ja.md](./04_roadmap.ja.md) を参照してください。

`libssruntime` は「**1 パート = 1 ドローコール**」が基本だった従来の per-part 描画モデルに加え、**「z-order 上で連続している同テクスチャ・同状態のパート群を 1 ドローコールに畳む」** ためのバッチ情報を `FrameData` 経由で公開します。

この章では、バッチ描画のための **データ契約**、**エンジン中立な描画ループの抽象アルゴリズム**、各エンジン (Godot / Unity / Unreal / WebGPU / WebGL2) における実装パターンを定義します。

---

## 1. 動機 (Motivation)

SpriteStudio のキャラクタは典型的に 30〜100 のパートで構成されます。シーン内に複数キャラを同時表示する場合、per-part 描画では：

- ドローコール数が **∑(パート数)** に比例
- 毎フレーム、エンジン側で transform / material / texture binding を per-part 更新

ドローコール数を減らすには、**同じ cellmap (テクスチャ) を参照し、ブレンドモードや状態が同じパート** をまとめて 1 ドローコールに発行するのが有効です。SS のキャラクタアセットは多くの場合 1〜数枚のアトラスを共有するため、典型的にバッチ数は 1〜10 程度に収束します。

ただし **z-order の正確性は維持しなければなりません**。`A → B → A → A` のような並びで A 同士を全部まとめると B との前後関係が崩れます。そのため SDK は **「z-order 上で連続している同状態パートだけを 1 グループにする RLE 方式」** を採用します。

---

## 2. データ契約 (Data Contract)

### 2.1 FrameData への追加

`framedata.fbs` の `FrameData` テーブルに 2 フィールドが追加されます:

```fbs
table FrameData {
  parts: [PartState];
  cells: [PartAttributeCell];
  parts_color: [PartAttributePartColor];
  shaders: [PartAttributeShader];
  vertices: [PartAttributeVertex];
  deforms: [PartAttributeDeform];

  // バッチ描画契約
  draw_order: [ushort];      // part_idx を描画順に並べた permutation
  draw_batches: [DrawBatch]; // 描画コマンド列 (RLE)
}
```

- `draw_order[r]` は **rank `r` の位置で描画される `part_idx`**。長さは `parts.size()` で、内容は `[0..N)` の置換。
- `draw_batches` は連続するバッチを順に並べたもの。`start_rank` / `count` で `draw_order` を切り出すと、そのバッチに含まれる part_idx 群が得られる。
- 描画は `draw_batches` を **配列順** に処理することで自動的に z-order に整合する。

### 2.2 DrawBatch struct

```fbs
struct DrawBatch {
  start_rank: ushort;     // draw_order の開始インデックス
  count: ushort;          // バッチ内のパート数
  texture_hash: uint32;   // cellmap.name_hash (FNV-1a) — テクスチャ不要なら 0
  vertex_count: ushort;   // バッチが必要とする総頂点数 (buffer 確保ヒント)
  index_count: ushort;    // 同 index 数
  blend_type: BlendType;  // 既存 BlendType enum
  kind: DrawBatchKind;
  flags: ubyte;           // 属性ヒント (2.4 節参照)
}
```

- `texture_hash` は SSAB の `cellmaps[i].name_hash` または `external_textures[i].name_hash` と一致。Player は Setup 時に作った `name_hash → Texture` テーブルで解決する。
- `vertex_count` / `index_count` は WebGPU/WebGL2 の事前バッファ確保サイズや Unity の `Mesh.SetVertexBufferParams` で活用できる。これがあれば毎フレームの buffer サイズ算出は不要。

### 2.3 DrawBatchKind

```fbs
enum DrawBatchKind: ubyte {
  Normal,    // PartTypeNormal の連続群 — バッチ可能
  Shape,     // PartTypeShape — count==1 (シングルトン)
  Instance,  // PartTypeInstance — count==1
  Effect,    // PartTypeEffect — count==1
  Mesh,      // PartTypeMesh — count==1 (将来実装)
  Text,      // PartTypeText — count==1 (将来実装)
  Nines,     // PartTypeNines — count==1 (将来実装)
  Mask,      // PartTypeMask — マスクスコープ境界 — count==1 (将来実装)
}
```

`Normal` 以外は **常に `count==1` のシングルトンバッチ** として吐かれます。Player は `kind` で switch して per-kind の処理パスにディスパッチします。

### 2.4 flags ビット定義

| bit | 名前 | 意味 |
|---:|---|---|
| 0 | `mask_influence` | バッチ内のパートが `PartData.mask_influence == true` (マスクの影響を受ける) |
| 1〜7 | (reserved) | 将来拡張用。現時点では 0 を読み取ること |

`mask_influence` をバッチキーに含めることで、マスクスコープ内の「マスクされる/されない」パートが混ざらず、Player のマスク処理挿入点が確定します。マスク実装が入るまでは Player 側でこのフラグを無視しても動作します (バッチが過剰分割される可能性はあっても描画は正しい)。

---

## 3. バッチング規則 (Batching Rules)

ランタイムが 2 パートを同一バッチにまとめるのは、以下の **すべて** が一致する場合のみ:

| キー要素 | ソース |
|---|---|
| `kind` | `PartData.part_type` の discriminator |
| `texture_hash` | `cell_texture_hashes[part_idx]` (cellmap 切替アニメ反映済み) |
| `blend_type` | `PartData.blend_type` |
| `flags::mask_influence` | `PartData.mask_influence` |

これら以外でも、以下は **強制バッチ境界**:

- **`PartTypeMask` パートが現れた点** (count=1 のシングルトン Mask バッチを emit)
- **`PartTypeShape` / `Instance` / `Effect` 等の非 Normal パート** (kind 別シングルトン化)

非表示 (`PartState.hide()`) のパートは `draw_batches` から **除外** されます (実描画されないため `draw_order` にも乗らない)。

### 3.1 静的最適化

`ZOrderTable.is_static && どのパートも cell_array を持たない` 場合、`draw_order` と `draw_batches` は setup 時に確定するためフレーム間でキャッシュされ、`get_frame_data` 呼び出しでは追加計算が発生しません。多くの SS アセットはこの条件を満たします。

---

## 4. 描画ループ契約 (Draw Loop Contract)

エンジン中立な抽象アルゴリズム:

```
let frame_data = decode_flatbuffer(get_frame_data(frame_no));
let order   = frame_data.draw_order;
let batches = frame_data.draw_batches;

for (b_idx, batch) in batches.enumerate() {
    let resource = ensure_resource_for(b_idx, batch.kind);
    apply_blend_state(resource, batch.blend_type);

    match batch.kind {
        Normal => {
            bind_texture(resource, batch.texture_hash);
            let vbuf = alloc_vertices(batch.vertex_count);
            let ibuf = alloc_indices(batch.index_count);
            let mut base = 0;

            for k in 0..batch.count {
                let part_idx = order[batch.start_rank + k];
                let world_m  = world_matrices[part_idx];
                let local_v  = local_vertices[part_idx];
                let local_uv = local_uvs[part_idx];
                let color    = part_color_for(part_idx);

                append_quad_or_pentagon(vbuf, ibuf, base,
                                        transform(world_m, local_v),
                                        local_uv, color);
                base += vertex_count_for(part_idx);
            }
            submit_draw_call(resource, vbuf, ibuf);
        }
        Shape    => draw_shape_singleton(order[batch.start_rank]),
        Instance => draw_instance_singleton(order[batch.start_rank]),
        Effect   => draw_effect_singleton(order[batch.start_rank]),
        Mask     => begin_or_update_mask_scope(order[batch.start_rank]),
        _ => { /* TODO: Mesh / Text / Nines */ }
    }
}
```

### 4.1 エンジンに任される設計選択

| 関心 | 選択肢 |
|---|---|
| world 変換 | CPU 側で頂点に焼く / vertex shader で uniform 経由 / instance attribute |
| バッチリソース | 永続プール / 毎フレーム再生成 / リングバッファ |
| マスク実装 | stencil / shader sampler / texture-based clipping |
| z-order の写像 | Godot `canvas_item z_index` / Unity `sortingOrder` / 単純な描画順 |
| バッファ更新方式 | `STREAM_DRAW` / persistent mapped / staging copy |

ssruntime は **これらに関与しません**。`world_matrices` と `local_vertices` を分離して提供する原則を維持し、Player の選択を奪わないようにします。

### 4.2 Mask scope セマンティクス (Phase 2)

`PartTypeMask` パートは、SS6 オリジナルランタイムと同じ **「z-order 上で expire するマスク」モデル** で処理します。本節は SS6 SDK の `Common/Animator/ssplayer_animedecode.cpp:1894-1989` (`SsAnimeDecoder::draw()`) を参照根拠としています。

#### モデル

- アニメーション中の **すべての mask 寄与パーツ** を z-order 順に並べた `mask_indices` リストを runtime が公開する。Phase 2 では `PartTypeMask` のみが対象 (Phase 3 で `Shape`/`Text`/`Nines` の `*_mask == true` を追加)。
- フレーム描画開始時、Player は **全 mask 寄与パーツをマスクバッファに描画** する (= 全マスクの和集合がアクティブな状態でループに入る)。
- z-order を進めて `kind=Mask` の DrawBatch に到達するたびに:
    1. `clear_mask_buffer()` でマスクバッファをクリア
    2. **そのマスクより後 (z-order 上で先に描画されないもの) のマスクのみ再描画**
    3. 結果として「現在のマスク」は「自分より z-order 後の mask 寄与パーツの和集合」に縮退する
- マスク以外のパートは `flags::mask_influence` を参照して、現在のマスクバッファをサンプル / 適用する。

つまりマスクは **MaskBegin/MaskEnd ペアではなく**、各 Mask パートが「自分自身のマスク寄与を取り除く z-order 上の単項イベント」として作用します。

> **【非表示マスクの扱い】** SS6 のコメント (`ssplayer_animedecode.cpp:1929-1933`) にある通り、`hide==true` の Mask パートも **マスクバッファ更新は実行する**必要があります (描画自体はスキップ)。Mask 処理を hide でスキップするとマスクバッファがクリアされず、後続の z-order に意図しないマスクが残ってしまいます。runtime はこれを `kind=Mask` のバッチを `hide==true` でも emit することで保証します (Normal バッチは hide 時に除外される、と対称的)。

#### Phase 2 で追加されるスキーマ

```fbs
table FrameData {
  // ... 既存フィールド
  draw_order: [ushort];
  draw_batches: [DrawBatch];

  // Phase 2 で追加
  mask_indices: [ushort];   // mask 寄与パーツの part_idx を z-order 順に並べたもの
}
```

#### Player の擬似コード

```
// フレーム描画開始時 — 全マスクをバッファに加算
for &part_idx in frame_data.mask_indices {
    render_mask_contribution(part_idx);
}

let mut mask_cursor = 0;
for batch in frame_data.draw_batches {
    if batch.kind == Mask {
        clear_mask_buffer();
        mask_cursor += 1;
        for &later_idx in &frame_data.mask_indices[mask_cursor..] {
            render_mask_contribution(later_idx);
        }
        continue;  // Mask パート自体は通常描画しない
    }

    // Normal/Shape/Instance/Effect の通常描画
    apply_blend_state(batch.blend_type);
    if batch.flags & FLAG_MASK_INFLUENCE != 0 {
        apply_mask_state(resource);   // バッファをサンプル / stencil read state ON
    } else {
        disable_mask_state(resource);
    }
    submit_draw_call(batch);
}
```

#### `mask_influence` の評価規則

`DrawBatch.flags` のビット 0 (`mask_influence`) は SSAB の `PartData.mask_influence` を反映します。SS6 では `state->maskInfluence` (`ssplayer_animedecode.cpp:333`) として親 Instance からの継承も含めて評価されます:

```
state.maskInfluence = part.maskInfluence && parent_decoder.maskParentSetting
```

本ランタイムでも同じセマンティクス:

- 通常パート: SSAB の `PartData.mask_influence` をそのまま flags に反映
- Instance 子プレイヤー内のパート: 親 Instance パートの `mask_influence` で AND 合成 (SS6 の `setMaskParentSetting` 相当)

これにより、SS6 のネスト Instance マスク継承挙動と一致します。

#### Phase 3 への拡張

`PartTypeShape` (`shape_mask==true`) / `Text` (`text_mask==true`) / `Nines` (`nines_mask==true`) も Phase 3 で `mask_indices` に追加します。これらは SS6 と同じく **「マスクバッファに寄与しつつ、自分自身も Normal 描画される」二重役割** を持ち、`draw_batches` には:

- `kind=Mask` (count=1) — マスクバッファ更新のトリガとして (hide 含む)
- ＋ `kind=Shape/Text/Nines` (count=1) — 通常描画として

の **2 バッチが同じ part_idx に対して emit** される、という構造になります。Phase 2 の `PartTypeMask` (純粋マスク = 通常描画なし) では Mask バッチ 1 つのみ emit。

---

## 5. エンジン別実装パターン

`DrawBatch` 1 個が以下にマップされる、というのが普遍的な基本構造です:

| エンジン | バッチ resource | 1 ドローコール |
|---|---|---|
| Godot | `canvas_item` 1 個 | `canvas_item_add_triangle_array` |
| Unity | `Mesh` + `MeshRenderer` 1 組 | `MeshRenderer` 描画 / `CommandBuffer.DrawMesh` |
| Unreal | `FProceduralMeshSection` または Slate `FBatchedElements` 1 セクション | セクション提出 |
| WebGPU | `GPUBuffer` (vertex+index) + `GPUBindGroup` | `encoder.drawIndexed` |
| WebGL2 | `WebGLBuffer` (vertex+index) + texture binding | `gl.drawElements` |

### 5.1 Godot (C++)

```cpp
RID ci = batch_ci_pool.ensure(b_idx);
rs->canvas_item_clear(ci);
rs->canvas_item_set_z_index(ci, b_idx);
rs->canvas_item_set_material(ci, get_blend_material(batch.blend_type)->get_rid());

PackedVector2Array verts; verts.resize(batch.vertex_count);
PackedVector2Array uvs;   uvs.resize(batch.vertex_count);
PackedColorArray   cols;  cols.resize(batch.vertex_count);
PackedInt32Array   inds;  inds.resize(batch.index_count);
// ... pre-transform on CPU, append per-part quads/pentagons ...

rs->canvas_item_add_triangle_array(ci, inds, verts, cols, uvs, {}, {},
                                   resolve_texture(batch.texture_hash)->get_rid());
```

### 5.2 Unity (C#)

```csharp
var mesh     = batchMeshPool.Ensure(bIdx);
var renderer = batchRendererPool[bIdx];

mesh.SetVertices(verts);   // Vector3[] (z = 0)
mesh.SetUVs(0, uvs);
mesh.SetColors(colors);
mesh.SetIndices(indices, MeshTopology.Triangles, 0);

renderer.material              = GetBlendMaterial(batch.blendType);
renderer.material.mainTexture  = ResolveTexture(batch.textureHash);
renderer.sortingOrder          = bIdx;
```

### 5.3 Unreal (C++)

```cpp
UProceduralMeshComponent* PMC = BatchPMCPool.Ensure(BIdx);
PMC->CreateMeshSection(0, Verts, Indices, /*Normals*/{}, UVs, Colors,
                       /*Tangents*/{}, /*CreateCollision*/false);
UMaterialInstanceDynamic* MID = GetBlendMID(Batch.BlendType);
MID->SetTextureParameterValue(TEXT("Tex"), ResolveTexture(Batch.TextureHash));
PMC->SetMaterial(0, MID);
```

### 5.4 WebGPU (TypeScript)

```ts
const vbuf = bufferPool.ensure(bIdx, batch.vertexCount * VERTEX_STRIDE);
const ibuf = bufferPool.ensureIndex(bIdx, batch.indexCount * 2);
device.queue.writeBuffer(vbuf, 0, vertexData);
device.queue.writeBuffer(ibuf, 0, indexData);

encoder.setPipeline(getPipeline(batch.blendType));
encoder.setBindGroup(0, getBindGroup(batch.textureHash));
encoder.setVertexBuffer(0, vbuf);
encoder.setIndexBuffer(ibuf, "uint16");
encoder.drawIndexed(batch.indexCount);
```

### 5.5 WebGL2 (TypeScript)

```ts
gl.bindBuffer(gl.ARRAY_BUFFER, vbufPool.ensure(bIdx));
gl.bufferData(gl.ARRAY_BUFFER, vertexData, gl.STREAM_DRAW);
gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ibufPool.ensure(bIdx));
gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indexData, gl.STREAM_DRAW);
gl.bindTexture(gl.TEXTURE_2D, resolveTexture(batch.textureHash));
applyBlendState(batch.blendType);
gl.drawElements(gl.TRIANGLES, batch.indexCount, gl.UNSIGNED_SHORT, 0);
```

---

## 6. シーングラフとの整合 (Scene Graph Integration)

### 6.1 Instance / Effect の親子関係

`PartTypeInstance` / `PartTypeEffect` は **シングルトンバッチ** のため、対応する resource (Godot canvas_item / Unity GameObject / Unreal Actor / Web は概念上のグループ) が一意に存在します。子プレイヤー / エフェクトコンテキストはそこに parent / attach すれば、スロットの transform / visibility を継承できます。

### 6.2 Y-sort 粒度 (該当エンジンのみ)

per-part から per-batch へ粒度が変わります。1 つの SS プレイヤーは「キャラクタ単位」で他の Y-sort オブジェクトと並べ替えられる挙動になります。SS キャラクタ内部のパートを他の Y-sort オブジェクトとインターリーブする用途は通常想定しません。

### 6.3 ヒットテスト

per-batch の resource は per-part 識別を持ちません。パートクリック判定が必要な場合は `world_matrices` + `local_vertices` を CPU 側で AABB / triangle test するメタ API を Player 側に別途実装してください (resource ベースのヒットテストは利用できません)。

---

## 7. 中立性ルール (Engine-Neutrality Rules)

`framedata.fbs` および `libssruntime` の FFI に **追加してはいけないもの**:

- エンジン固有のハンドル型 (`RID`, `MeshId`, `Texture2D` 参照など)
- エンジン固有の z-order 値域 (`z_index` の int8 範囲、`sortingOrder` の int16 範囲など)
- シーングラフの親子関係 (SSAB の `parent_index` はキャラ階層であってレンダリング階層ではない)
- pre-transform 済みワールド座標 (Player の選択を奪う — `local_vertices` + `world_matrices` を分離して提供する原則を維持)
- エンジン固有のシェーダ ID / マテリアル ID

新規 FFI を追加する際は、**「全 5 エンジンで同じ抽象として扱えるか」** を必ずチェックしてください。

---

## 8. 実装ロードマップ

| Phase | スコープ | スキーマ変更 |
|---:|---|---|
| 1 | `Normal` バッチ + `texture_hash` / `blend_type` キー | `DrawBatch`, `DrawBatchKind`, `draw_order` / `draw_batches` 追加 |
| 2 | `PartTypeMask` の expire-on-encounter scope + `mask_influence` フラグ | `FrameData.mask_indices` 追加、`flags` ビット 0 有効化 |
| 3 | `Shape`/`Text`/`Nines` の `*_mask` を `mask_indices` に追加 (二重役割パーツ) | (スキーマ不変、計算ロジック拡張のみ) |
| 4 | `Shader` キー (per-frame アニメ対応) | `flags` ビット拡張 |
| 5 | `Mesh` / `Text` / `Nines` kind 実装 | 既存 kind に処理を実装 |

破壊的変更が許容される pre-release 期間中に Phase 1〜2 を完了する想定です。Phase 移行ごとに本章の対応セクションを更新してください。

---

## 9. 関連章

- 描画ループの基本: [07_render_loop.ja.md](./07_render_loop.ja.md)
- 拡張機能 (Instance / Effect / PartsColor): [08_advanced_rendering.ja.md](./08_advanced_rendering.ja.md)
- 実装ロードマップ: [04_roadmap.ja.md](./04_roadmap.ja.md)
- アーキテクチャと中立性: [02_architecture.ja.md](./02_architecture.ja.md)

---

[目次に戻る](./README.ja.md)
