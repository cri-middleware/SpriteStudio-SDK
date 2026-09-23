[**English**](./SCHEMA.md) | [**日本語**](./SCHEMA.ja.md)

# SS-Runtime FrameData スキーマ解説

このドキュメントでは、SS-Runtime で使用される `FrameData` スキーマ (`framedata.fbs`) について解説します。`FrameData` は、SS-Runtime コアとプレイヤー（レンダリングエンジン）の間で、FFI を介してデータをやり取りするための主要な**データ交換インターフェース**として機能します。

## 概要: データが生成されるタイミングと内容

`FrameData` を理解するために、オリジナルの SpriteStudio データからランタイム出力までのデータの流れ（パイプライン）を見てみましょう。

`.ssab` はコンバート段階で**オフライン**で生成され、キーフレームのタイムライン全体を含んでいますが、`FrameData` は**実行時（ランタイム）に動的に生成**され、単一フレームのスナップショットのみを含みます。

具体的には、`FrameData` は `.ssab` ファイルから `AnimationData` を受け取り、特定のフレーム番号でキーフレームを評価・補間し、すべてのパーツの `PartState` を出力します。

```text
[オフライン コンバート]
.ssae (アニメーションパック) ──────► .ssab (SsAnimeBinary)

[ランタイム 再生時]
.ssab (SsAnimeBinary)
 │
 ├─ AnimationData
 │    └─ parts_animation_data: [PartAnimationData] (キーフレーム配列: POSX, ROTZ など)
 │
 ＋  現在のフレーム番号 (例: Frame 15)
 │
 ▼  (SS-Runtime: 補間計算 & 座標系変換)
 │
FrameData (FFI を介したメモリバッファ)
 └─ parts: [PartState] (Frame 15 における評価結果: position_x, rotation_z など)
```

### 主な特徴

| 項目 | SSAB (`SsAnimeBinary`) | FrameData (`FrameData`) |
|------|------------------------|-------------------------|
| **生成タイミング** | オフラインで生成 (アセット) | 実行時に生成 (フレームごと) |
| **ソースデータ**| `.ssae`, `.ssce`, `.ssee` など | `.ssab` 内の `AnimationData` |
| **データ内容** | キーフレームの配列 (疎なデータ) | 単一フレームの評価済み値 (密なデータ) |
| **単位** | アニメーションパックごと | フレームごと |

---

## 属性マッピング: SSAB から PartState へ

以下の表は、SSAB の `PartAnimationData` のどの配列が評価され、`FrameData` の `PartState` のフィールドに格納されるかを正確に示しています。

| PartState フィールド | ソース: SSAB `PartAnimationData` 配列 | 備考 |
|:---|:---|:---|
| `position_x/y/z` | `position_x/y/z_array` | 補間済み座標 (ターゲット座標系へ変換済み)。 |
| `rotation_x/y/z` | `rotation_x/y/z_array` | 補間済み回転角 (度数法)。 |
| `scale_x/y` | `scale_x/y_array` | 補間済みスケール倍率。 |
| `local_scale_x/y` | `local_scale_x/y_array` | 補間済みローカルスケール倍率。 |
| `alpha` | `alpha_array` | 補間済み不透明度 (0.0 ～ 1.0)。 |
| `local_alpha` | `local_alpha_array` | 補間済みローカル不透明度。 |
| `priority` | `priority_array` | 現在の Z オーダー (優先度)。 |
| `flip_h/v` | `flip_h/v_array` | 現在の真偽値反転フラグ。 |
| `hide` | `hide_array` | 現在の非表示フラグ。 |
| `pivot_x/y` | `pivot_x/y_array` | 補間済みピボット点 (回転・スケール中心)。 |
| `anchor_x/y` | `anchor_x/y_array` | 補間済み配置アンカー。 |
| `size_x/y` | `size_x/y_array` | 補間済みの物理サイズ。 |
| `img_flip_h/v` | `img_flip_h/v_array` | 現在の画像のみ反転フラグ。 |
| `uv_translation_x/y`| `uv_translation_x/y_array`| 補間済みテクスチャ UV オフセット。 |
| `uv_rotation_z` | `uv_rotation_z_array` | 補間済み UV 回転。 |
| `uv_scale_x/y` | `uv_scale_x/y_array` | 補間済み UV スケール。 |
| `bounding_radius` | `bounding_radius_array` | 補間済み当たり判定半径。 |
| `mask` | `mask_array` | 補間済みマスク影響度。 |
| `skew_x/y` | `skew_array` | SSAB の Vec2 歪み補間から算出。 |
| `texture` | `texture_array` | テクスチャ変更（`TCHG`）: テクスチャ名の fnv1a ハッシュで、`external_textures` から解決可能。`0` は上書き無し（パーツは自身のセルのテクスチャを使用）。ステップ評価であり補間されない。`DrawBatch.texture_hash` に反映済みのため、`texture_hash` でバッチ処理する Player はパーツ単位の対応を追加する必要はない。 |

---

## `PartState` の同定

`PartState.part_index` は、ソース SSAB の `parts` 配列（および `AnimationData.parts_animation_data`）上のインデックスです。現在のランタイムは `FrameData.parts` を SSAB と同じ順序で出力するため `parts[i].part_index() == i` ですが、ID の権威は `part_index` 側として扱ってください — `EventsPerFrame.users[i].part_index` などのイベントペイロードは SSAB パーツインデックス空間を参照しており、それらを `FrameData` に結びつけるのがこのフィールドです。

---

## 複雑な属性のマッピング（プーリング）

複雑なデータ構造の重複を避けるため、複雑な属性は `FrameData` 内のプールに格納されます。これらは `PartState` 内で `short` 型のインデックスとして参照されます。インデックスが `-1` の場合は「このフレームにこの属性のエントリは無い」ことを意味します。

| PartState 参照フィールド | ソース: SSAB `PartAnimationData` 配列 | FrameData 内のテーブル | 説明 |
|:---|:---|:---|:---|
| `cell` | `cell_array` | `cells` | セルマップとセル名ハッシュへの参照。通常描画に使用されるテクスチャはここから取得されます。 |
| `part_color` | `part_color_array` | `parts_color` | 4 頂点ぶんのカラーとブレンドタイプ。**RGBA の `a` チャンネルにはパーツの階層的 `alpha` が事前合成済み** なので、Player 側で再度アルファを掛ける必要はありません。 |
| `shader` | `shader_array` | `shaders` | 補間済みのシェーダーパラメータとマップ ID。 |
| `vertex` | `vertex_array` | `vertices` | **算出済みのローカル頂点座標**。4 隅 (`lt`/`rt`/`lb`/`rb`) は常に `pivot` / `size` / `img_flip_h/v` および補間済みの raw deform オフセット適用済みで、Player はそのままワールド行列を掛けて描画できます。5 番目の `center` 頂点は **`update_flag` に `AttributeVertex` か `AttributePartColor` が立っているときのみ** 設定されます (頂点変形やパーツカラーのグラデーションを center 起点の三角形ファンで滑らかに描画するため)。それ以外のケースでは `center = (0, 0)` で、Player は 4 隅から 2 三角形を組むだけで描画できます。生の deform オフセットは公開されません。`update_flag` 内の `AttributeVertex` フラグは、ソース SSAB に実際に頂点変形アニメがあるかどうかを引き続き示します。 |
| `deform` | `deform_array` | `deforms` | 補間済みのメッシュ変形座標。 |

---

## 描画プラン: 順序・バッチ・マスクスライス

`FrameData` はパーツ単位の状態だけでなく、**エンジン非依存の描画プラン** も保持します。事前にソートされたパーツ並びと、それを連続スライスに切り出したバッチリストを Player が 1 フレームに 1 度だけ走査します。これが「DrawBatch コントラクト」です。消費側の正典は [Batched Rendering Contract](./docs/30_core/40_render_loop.ja.md) です。

| FrameData フィールド | 型 | 説明 |
|:---|:---|:---|
| `draw_order` | `[ushort]` (required) | 描画順に並んだパーツ。値は **`FrameData.parts` のインデックス** （`part_index` ではない）。非表示・マスクで除外されたパーツなど、描画対象外のパーツは含まれません。 |
| `draw_batches` | `[DrawBatch]` (required) | マージ済みの描画コマンド列。各バッチは `draw_order` の連続スライスを覆い、スライス内で描画種別・ブレンドモード・テクスチャが共通になります。Player は バッチごとに 1 ドローコールを発行します。 |
| `mask_indices` | `[ushort]` (required) | そのフレームの **pure マスク**（`PartTypeMask` および `draw_as_mask`）の **`FrameData.parts` インデックス** を、`draw_order` と同じ順に並べたもの。**writer リストではありません**: クリッピング (`mask_write`) を含まず、`Hide == true` のマスクも落とさないため `draw_order` と一致しません。writer の確定は `draw_order` を走査して行ってください — [マスク §3](./docs/40_parts/40_mask.ja.md#3-framedata-から読むもの) 参照。 |

### `DrawBatch`

インライン `struct` (固定サイズ、vtable なし)。`draw_batches` 上の位置で参照されます:

| フィールド | 型 | 説明 |
|:---|:---|:---|
| `start_rank` | ushort | このバッチが指す `draw_order` の開始インデックス。 |
| `count` | ushort | このバッチに含まれる `draw_order` 連続エントリ数。 |
| `texture_hash` | uint32 | バッチがバインドするテクスチャの fnv1a ハッシュ。通常は `cellmap_name_hash`（`.ssce` を除いたセルマップ名）だが、スライス内のパーツがテクスチャ変更（`TCHG`）で上書きされている場合はそのテクスチャ *名* のハッシュ。TCHG の上書き値はバッチキーに含まれるため、同一セルマップを参照していてもバインド先テクスチャが異なるパーツは別バッチに分割される。Player は両方のハッシュ空間（セルマップ名と `external_textures` エントリ）を解決する必要がある（TCHG が指すテクスチャ名は必ず `external_textures` に含まれる）。テクスチャ不要な種別（`Shape` 等）では `0`。 |
| `vertex_count` | ushort | スライス内の全パーツが寄与する頂点数の合計。 |
| `index_count` | ushort | スライス内の全パーツが寄与するインデックス数の合計。 |
| `blend_type` | BlendType | スライス共通の描画先との合成モード。ssab の `PartData.blend_type`。パーツカラーの `blend_type`（テクスチャとの合成）とは別。 |
| `kind` | DrawBatchKind | 描画カテゴリ（下表）。Player がスライス内パーツについてどのジオメトリ／データを取り出すかを決定。 |
| `flags` | ubyte | バッチごとのフラグビット。現状定義済み: ビット `0x01` = `MASK_INFLUENCE` (このスライス内のパーツがアクティブなマスクスライスでマスクされる)。残りビットは予約済み — 未知ビットは無視すること。完全な契約は [Batched Rendering Contract](./docs/30_core/40_render_loop.ja.md) を参照。 |

### `DrawBatchKind`

| バリアント | パーツごとのジオメトリソース |
|:---|:---|
| `Normal` | 標準スプライト矩形 — `vertex` プール + `cell` プール（テクスチャは `texture_hash` 経由: `cellmap_name_hash`、または上書き時は TCHG のテクスチャ名）。 |
| `Shape` | `PartTypeShape` から生成される手続き的シェイプ（テクスチャ無し）。 |
| `Instance` | サブアニメのインスタンススロット — スロット pull の詳細は [Player Guide](./docs/README.ja.md) を参照。 |
| `Effect` | 該当 `EffectSlot` のパーティクル矩形（`effect_draw_plan.fbs` 参照）。 |
| `Mesh` | `PartMeshBinding` によるボーンスキニング済みインデックス付きメッシュ。 |
| `Text` | テキストパーツの描画順における位置。1 パーツ 1 バッチで、ジオメトリ（`vertex_count` / `index_count` は `0`）もテクスチャ（`texture_hash`）も持たない。字詰め・ラスタライズ・フォント解決は Player の担当（[制限事項](./docs/10_introduction/20_limitations.ja.md) を参照）。マスク指定されたテキストパーツはプランから除外される。 |
| `Nines` | `PartTypeNines` から生成した 9 パッチのグリッド。頂点・UV・インデックスは Nines 専用 SoA バッファにパーツ単位で入る（[9 スライス・シェイプ・テキスト](./docs/40_parts/30_special_parts.ja.md) を参照）。 |
| `Mask` | ステンシル書き込み専用バッチ。色出力は行わず、対象パーツは `mask_indices` にも列挙される。 |

> **プランの走査:** `draw_batches` の各 `batch` について、`draw_order[batch.start_rank .. batch.start_rank + batch.count]` をイテレートし、`PartState` を取り出して `batch.kind` のジオメトリを発行する。GPU ステート（テクスチャ・ブレンド）の切替はパーツごとではなくバッチごとに 1 回だけ行う。

---

## 関連スキーマ: `EffectDrawPlan`

エフェクトレンダラーは独立したルートバッファ（`effect_draw_plan.fbs`、ルート `EffectDrawPlan`）で提供され、`EffectSlot::step` / `ss_effect_slot_step` から返されます。`FrameData` からは `DrawBatchKind::Effect` バッチを通じて間接的に参照されるのみで、実際の頂点ジオメトリは `EffectDrawPlan` 側に乗ります。同じフレームの `FrameData` と組で消費する想定です。

| フィールド | 型 | 説明 |
|:---|:---|:---|
| `commands` | `[EffectDrawCommand]` (required) | エミッタごとの描画コマンド。`cellmap_hash`, `quad_count`, `particle_offset`, `blend` (`0=Mix`, `1=Add` — `EmitterState.blend_type` の生値) を保持。 |
| `vert_stride` | ubyte | 2D ネイティブエンジン (Godot, pixi.js) では `2`、XYZ 頂点が必要なエンジン (Unity Mesh, Unreal FProcMeshVertex) では `3`。`step` 呼び出しごとに選択。 |
| `verts` | `[float]` (required) | 長さは `total_quads * 4 * vert_stride`。Y フリップは呼び出し側の要求に応じて適用済み。 |
| `uvs` | `[float]` (required) | 長さは `total_quads * 4 * 2`。 |
| `colors` | `[float]` (required) | 長さは `total_quads * 4 * 4` (RGBA, 0..1 に正規化済み)。 |
| `indices` | `[int]` (required) | 長さは `total_quads * 6`。値は **quad ローカル** (`0..quad_count*4`)。エンジンはスロット相対の verts と共にスライスを渡すだけでよく、再ベースは不要。 |

コマンド `c` のスライス計算 (`vs = vert_stride`):

```
verts  : c.particle_offset * 4 * vs .. (c.particle_offset + c.quad_count) * 4 * vs
uvs    : c.particle_offset * 4 * 2  .. (c.particle_offset + c.quad_count) * 4 * 2
colors : c.particle_offset * 4 * 4  .. (c.particle_offset + c.quad_count) * 4 * 4
indices: c.particle_offset * 6      .. (c.particle_offset + c.quad_count) * 6
```

---

## 更新フラグ (`UpdateAttributeFlags`)

`PartState` の `update_flag` は、現在のフレームにおいて、SSAB ソースデータでどのアトリビュートが明示的にアクティブであるか、または変更されたか（セットアップ値またはキーフレーム経由）を示すビットマスクを提供します。これにより、Player はデフォルト属性の処理をスキップできます。

| フラグ名 | ソース: SSAB `PartAnimationData` 配列 |
|:---|:---|
| `AttributeCell` | `cell_array` |
| `AttributePosX/Y/Z` | `position_x/y/z_array` |
| `AttributeRotX/Y/Z` | `rotation_x/y/z_array` |
| `AttributeSclX/Y` | `scale_x/y_array` |
| `AttributeLosclX/Y` | `local_scale_x/y_array` |
| `AttributeAlpha` | `alpha_array` |
| `AttributeLoAlpha` | `local_alpha_array` |
| `AttributePrio` | `priority_array` |
| `AttributeFlipH/V` | `flip_h/v_array` |
| `AttributeHide` | `hide_array` |
| `AttributePartColor` | `part_color_array` |
| `AttributeShader` | `shader_array` |
| `AttributeVertex` | `vertex_array` |
| `AttributePivotX/Y` | `pivot_x/y_array` |
| `AttributeAnchorX/Y` | `anchor_x/y_array` |
| `AttributeSizeX/Y` | `size_x/y_array` |
| `AttributeImgFlipH/V` | `img_flip_h/v_array` |
| `AttributeUvtX/Y` | `uv_translation_x/y_array` |
| `AttributeUvrZ` | `uv_rotation_z_array` |
| `AttributeUvsX/Y` | `uv_scale_x/y_array` |
| `AttributeBoundr` | `bounding_radius_array` |
| `AttributeMask` | `mask_array` |
| `AttributeSkew` | `skew_array` |
| `AttributeDeform` | `deform_array` |
| `AttributeTexture` | `texture_array` |
