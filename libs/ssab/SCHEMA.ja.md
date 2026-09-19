[**English**](./SCHEMA.md) | [**日本語**](./SCHEMA.ja.md)

# SSAB スキーマドキュメント

本ドキュメントでは、SSAB (SpriteStudio Anime Binary) および SSQB (SpriteStudio Sequence Binary) のスキーマを、元の XML ベースの ss*e フォーマット (ssae, ssce, ssee, ssse, ssqe) との比較を通じて説明します。

## 概要

| 項目 | ss*e (XML) | ssab / ssqb (バイナリ) |
|------|-----------|---------------------|
| フォーマット | XML (quick-xml + serde) / BMFont テキスト | FlatBuffers |
| 文字列検索 | 名前ベースの文字列比較 | ハッシュベース (uint32) の検索 |
| アクセスパターン | 所有権付き構造体へのデシリアライズ | ゼロコピー読み込み |
| `no_std` サポート | なし | あり |

### 変換単位

`.ssab` ファイルは **`.ssae`（アニメーションパック）ごとに1つ** 生成されます。`.ssab` には ssae のデータに加え、ssae が参照する ss*e データ (ssce, ssee, ssse, fnt) が含まれます。`.sspj` プロジェクトファイル自体は ssab に含まれません — 変換時にリソース参照の解決のためだけに使用されます。

```
.sspj (プロジェクト) — リソース解決にのみ使用
 │
 ├─ .ssae (アニメーションパック A) ──────► .ssab (A 用の SsAnimeBinary)
 │    含まれるデータ:
 │    ├─ .ssae のアニメーション & パーツデータ
 │    ├─ .ssce (A が使用するセルマップ)
 │    ├─ .ssee (A が使用するエフェクト)
 │    ├─ .ssse (A が使用するサウンド)
 │    └─ .fnt  (A が使用するフォント)
 │
 ├─ .ssae (アニメーションパック B) ──────► .ssab (B 用の SsAnimeBinary)
 │    含まれるデータ:
 │    ├─ .ssae のアニメーション & パーツデータ
 │    └─ .ssce (B が使用するセルマップ)
 │
 └─ .ssqe (シーケンスパック)      ──────► .ssqb (SsSequenceBinary)
```

各 `.ssab` は自己完結的です。特定のアニメーションパックが参照する ss*e リソースのみが埋め込まれ、使用されないリソースは除外されます。

---

## SSAB ルート: `SsAnimeBinary`

**FlatBuffers 識別子:** `"SSAB"`

| ssab フィールド | 型 | ソース | 備考 |
|---------------|------|------|------|
| `version` | uint | — | バイナリフォーマットバージョン (0x00000001) |
| `name` | string | `SsAnimePack.name` (ssae) | アニメーションパック名 |
| `name_hash` | uint32 | — | 高速検索用のハッシュ |
| `parts` | [PartData] | `SsAnimePack.model.part_list` (ssae) | このパック内の全パーツのフラット配列 |
| `animations` | [AnimationData] | `SsAnimePack.anime_list` (ssae) | このパック内の全アニメーション |
| `cellmaps` | [CellMap] | `SsCellMap` (ssce) | このパックが参照するセルマップのみ（エフェクトが使用するものも含む） |
| `effects` | [Effect] | `SsEffectFile` (ssee) | Effect タイプのパーツが参照するエフェクトのみ |
| `sound_lists` | [SoundList] | `SsAudioPack` (ssse) | パックに Audio タイプのパーツが含まれる場合のみ |
| `font_bitmaps` | [FontBitmap] | `SsCharMap` (fnt) | `text_bitmap == true` の Text タイプパーツが使用するビットマップフォントのみ |
| `external_textures` | [ExternalTexture] | — | 外部テクスチャ参照。`name_hash` でソート。プロジェクトの `<ExternalTextures>` リストと、この pack 内の `TCHG` キーが指す全ての `TextureName`（`common.png` のようなセルマップ画像を含む）の和集合。Player が `TCHG` のハッシュ（ひいては `DrawBatch.texture_hash`）をロード可能な名前へ逆引きするためのテーブルであり、両方の供給元を網羅する必要がある。 |
| `external_instances` | [ExternalInstance] | — | クロスパックインスタンス参照。`(anime_pack_name_hash, anime_name_hash)` でソートされる。このテーブルに FlatBuffers の `(key)` は無いため、`LookupByKey` ではなくベクタを走査して引く。各エントリは `anime_pack_name_hash`（参照先 pack の `SsAnimeBinary.name_hash` と一致）、`anime_pack_name`（隣接する `<pack>.ssab` ファイル探索に使用）、`anime_name_hash`、`anime_name`（診断用）を持つ。 |
| `embedded_assets` | [EmbeddedAsset] | — | オプションで埋め込まれるバイナリアセット（画像、サウンドなど） |

---

## パーツデータ

### `PartData` (ssab) vs `SsPart` (ssae)

| ssab フィールド | 型 | ssae フィールド | 備考 |
|---------------|------|---------------|------|
| `name` | string | `name` | パーツ名 |
| `parent_index` | short | `parent_index` | 親パーツのインデックス |
| `bounds_type` | BoundsType | `bounds_type` | バウンディング計算方式 |
| `blend_type` | BlendType | `alpha_blend_type` | フィールド名変更 |
| `visible_inside_mask` | bool | `visible_inside_mask` | 被マスク側の描画パーツの極性。`1` = マスクの内側に描画 / `0` = 外側に描画 → [Mask の実装 §2-4](../ssruntime/docs/40_components/40_mask.ja.md#2-4-極性マスクの内側か外側のどちらに描画するか) |
| `mask_write` | bool | `write_mask` | フィールド名変更。描画パーツを **クリッピング** の writer にするフラグ。クリッピングには `DrawBatchKind::Mask` が出ないため、writer の確定はこのフラグから行う → [Mask の実装 §1-2](../ssruntime/docs/40_components/40_mask.ja.md#1-2-パーツの役割と-mask_influence) |
| `mask_influence` | bool | `mask_influence` | **役割で意味が変わる二重定義**。マスク writer では OP（`0` = 増分 / `1` = 反転）、被マスク側の描画パーツでは対象フラグ（`1` = 適用される / `0` = オプトアウト）。クリッピングは writer 兼描画パーツなので、同じ値が両方の意味で同時に使われる。片方の意味を捨てないこと → [Mask の実装 §1-2](../ssruntime/docs/40_components/40_mask.ja.md#1-2-パーツの役割と-mask_influence) |
| `draw_as_mask` | bool | `shape_mask` / `text_mask` / `nines_mask` | 描画パーツ (Shape/Text/Nines) の pure マスク化フラグを統合。ssae では型ごとに排他の別タグ。`PartTypeMask` は常に pure マスクなので本フラグに依らない |
| `output_influence` | bool (既定 `true`) | `output_influence` | 作画データからそのまま引き継がれる。`libssruntime` は読まない |
| `mesh_binding` | PartMeshBinding | (変換時生成) | `PartTypeMesh` 用の頂点ごとのボーン影響値。下記 [メッシュバインディング](#メッシュバインディング) 参照。メッシュ以外のパーツでは省略。 |
| `part_type` | PartType (union) | `part_type` + 型固有フィールド | 後述 |

### パーツタイプ Union

ssae ではパーツタイプは単純な列挙型 (`SsPartType`) であり、型固有のデータ（shape, text, nines など）は `SsPart` のフラットなフィールドとして格納されます。ssab では FlatBuffers の **union** (`PartType`) として型ごとに専用テーブルを用いてモデル化されています:

| ssab Union バリアント | ssae 対応 | データ |
|---------------------|----------|------|
| `PartTypeNull` | `SsPartType::Null` | なし |
| `PartTypeNormal` | `SsPartType::Normal` | なし |
| `PartTypeShape` | `SsPartType::Shape` | `shape_type` |
| `PartTypeText` | `SsPartType::Text` | `text`, `text_bitmap`, `text_family` 等 |
| `PartTypeNines` | `SsPartType::Nines` | `nines_margin_l/r/t/b`, `nines_fill_mode` |
| `PartTypeInstance` | `SsPartType::Instance` | `ref_anime_pack` + `ref_anime_pack_hash`、`ref_anime_name` + `ref_anime_name_hash` (ssae: `ref_anime_pack` + `ref_anime` を文字列で保持) |
| `PartTypeArmature` | `SsPartType::Armature` | なし |
| `PartTypeEffect` | `SsPartType::Effect` | `ref_effect_name_hash` (ssae: `ref_effect_name` を文字列で保持) |
| `PartTypeMesh` | `SsPartType::Mesh` | なし |
| `PartTypeMovenode` | `SsPartType::Movenode` | なし |
| `PartTypeConstraint` | `SsPartType::Constraint` | なし |
| `PartTypeMask` | `SsPartType::Mask` | なし |
| `PartTypeJoint` | `SsPartType::Joint` | なし |
| `PartTypeBonepoint` | `SsPartType::Bonepoint` | なし |
| `PartTypeTransformConstraint` | `SsPartType::TransformConstraint` | なし |
| `PartTypeCamera` | `SsPartType::Camera` | なし |
| `PartTypeAudio` | `SsPartType::Audio` | なし |

### パック・モデルレベルで引き継がれないフィールド

以下の ssae パック・モデルレベルのフィールドは ssab に含まれません:

- `SsAnimePack.version` (ssab は独自の `version` フィールドを持つ)
- `SsAnimePack.settings` (パックレベルのアニメーション設定。各アニメーションの `settings` は変換される)
- `SsAnimePack.cellmap_names` (ssab では実データとして埋め込まれるため不要)
- `SsModel.bone_list` (ボーン名→インデックスのマップ、エディタ/コンバータ用)
- `SsModel.ref_setup_anime_pack`, `SsModel.ref_setup_animation` (ロード時に計算される値)

### パーツレベルで引き継がれないフィールド

以下の ssae `SsPart` フィールドはエディタ専用のため、ssab には含まれません:

- `array_index`, `show`, `locked`, `color_label` (エディタ UI の状態)
- `inherit_type`, `inherit_rates` (変換時に解決済み)
- `bone_length`, `bone_position`, `bone_rotation` (ボーン編集データ)
- `weight_position`, `weight_impact`, `mesh_weight_type`, `mesh_weight_strong` (メッシュウェイト編集データ)
- `ik_depth`, `ik_rotation_arrow` (IK ソルバーパラメータ)

### メッシュバインディング

`PartTypeMesh` の場合、コンバータは ssae のメッシュウェイト記述を頂点単位のボーン影響値に解決し、`PartData.mesh_binding` に格納します。それ以外のパーツタイプではこのフィールドは省略されます。影響値は入れ子のテーブルではなく、**プレフィックスオフセット表で引く並行配列** として保持されます（頂点ごとではなくパーツごとに 1 回のベクタ確保で済む）。

| フィールド | 型 | 備考 |
|----------|----|------|
| `vertex_influence_offset` | [uint32] | プレフィックスオフセット。メッシュ頂点1つにつき1エントリに **末尾の合計値** を加えた `頂点数 + 1` 要素。頂点 `i` の影響値は下記 4 配列の半開区間 `[offset[i], offset[i + 1])`。外側のインデックスはメッシュ頂点の並びと一致。 |
| `influence_bone` | [ushort] | 影響を与えるボーンのパーツインデックス。ssae のボーンリスト上のインデックスから対応するパーツインデックスに変換済みで、ランタイムはそのまま `parts` を引ける。 |
| `influence_weight` | [float] | 影響度。`0.0`〜`1.0` に正規化済み（ssae では百分率）。 |
| `influence_coord_x` / `influence_coord_y` | [float] | バインドポーズ時の、影響ボーンからの頂点相対オフセットを 2 配列に分けたもの。 |

---

## セルマップ

### `CellMap` (ssab) vs `SsCellMap` (ssce)

| ssab フィールド | 型 | ssce フィールド | 備考 |
|---------------|------|---------------|------|
| `name_hash` | uint32 (key) | — | 高速検索用ハッシュ |
| `name` | string | `name` | セルマップ名 |
| `image_path` | string | `image_path` | ソース画像のパス |
| `image_path_hash` | uint32 | — | 画像パスのハッシュ |
| `size_original` | Vec2 | `pixel_size` | 画像のピクセルサイズ |
| `wrap_mode` | TexWrapMode | `wrap_mode` | テクスチャラップモード |
| `filter_mode` | TexFilterMode | `filter_mode` | テクスチャフィルタモード |
| `cells` | [Cell] | `cells` | セル定義 |

### `Cell` (ssab) vs `SsCell` (ssce)

| ssab フィールド | 型 | ssce フィールド | 備考 |
|---------------|------|---------------|------|
| `name_hash` | uint32 (key) | — | 高速検索用ハッシュ |
| `name` | string | `name` | セル名 |
| `rectangle` | Rect | `pos` + `size` | そのまま格納。`x1,y1` = `pos`、`x2,y2` = `size` で、右下座標ではなく幅・高さです |
| `pivot` | Vec2 | `pivot` | セルのピボットポイント |
| `rotated` | bool | `rotated` | アトラス内でセルが回転しているか |
| `table_coord_x` | [float] | `mesh_point_list[*].x` | メッシュ頂点 X 座標 (SoA) |
| `table_coord_y` | [float] | `mesh_point_list[*].y` | メッシュ頂点 Y 座標 (SoA) |
| `table_index_vertex` | [ushort] | `mesh_tri_list` | メッシュ三角形インデックス（SsPoint3 からフラット化） |

### 引き継がれないフィールド

- `version`, `override_tex_settings` (ssce メタデータ)
- `is_mesh`, `inner_point`, `outer_point` (メッシュ編集データ)
- `div_type`, `div_w`, `div_h` (メッシュ分割パラメータ)

---

## アニメーションデータ

### `AnimationData` (ssab) vs `SsAnimation` (ssae)

| ssab フィールド | 型 | ssae フィールド | 備考 |
|---------------|------|---------------|------|
| `name` | string (required) | `name` | アニメーション名。`(required)` で verify 時に存在を強制するが `(key)` は付けない: 配列は sspj のソース順序を保持して "最初のアニメーション" の意味を決定的にする必要があり、`LookupByKey` のためのキー順ソートと両立しないため。 |
| `name_hash` | uint32 | — | fnv1a(name)。ハッシュベースのルックアップはリニアスキャンで行う。 |
| `start_frame` | short | `settings.start_frame` | 開始フレーム |
| `end_frame` | short | `settings.end_frame` | 終了フレーム |
| `total_frame` | short | `settings.frame_count` | 総フレーム数 |
| `fps` | short | `settings.fps` | フレームレート |
| `canvas_size` | Vec2 (required) | `settings.canvas_size` | オーサリングされたキャンバスサイズ (px)。 |
| `pivot` | Vec2 (required) | `settings.pivot` | キャンバス内でのランタイム原点の位置 (正規化)。既定は `0, 0` (キャンバス中心)。 |
| `parts_initial_attributes` | [InitialPartAttributes] | セットアップアニメデータ | 初期属性のセクション参照 |
| `parts_animation_data` | [PartAnimationData] | `part_animes` | アニメーションデータレイアウトのセクション参照 |
| `events` | [EventsPerFrame] | パーツごとのイベント属性 | イベントのセクション参照 |
| `initial_events` | [InitialEvents] | セットアップアニメのイベント | 初期イベント状態 |
| `labels` | [Label] | `labels` | アニメーションラベル |

### 引き継がれないフィールド

- `override_settings` (エディタ設定)
- `is_setup` (セットアップアニメのデータは `parts_initial_attributes` に統合)

---

## アニメーションデータレイアウト: カラム型 vs 行型

これは2つのフォーマット間の最も重要な構造的違いです。

### ssae: 行型（パーツごと・属性ごと）

ssae ではアニメーションデータがパーツごと、次に属性ごとの階層構造で整理されています:

```
SsAnimation
  └─ part_animes: [SsPartAnime]       // パーツごとに1つ
       ├─ part_name: "arm"
       └─ attributes: [SsAttributeKind]  // 属性タイプのリスト
            ├─ PosX { keys: [{ time: 0, value: 10.0 }, { time: 30, value: 50.0 }] }
            ├─ RotZ { keys: [{ time: 0, value: 0.0 }, { time: 15, value: 90.0 }] }
            └─ Alpha { keys: [{ time: 0, value: 1.0 }] }
```

各属性はタグ文字列（例: `"POSX"`, `"ROTZ"`, `"ALPH"`）で識別されます。

### ssab: カラム型（パーツごと・属性タイプ別の配列）

ssab では各パーツのアニメーションデータに、属性ごとの**型付き配列**の固定セットが含まれます:

```
AnimationData
  └─ parts_animation_data: [PartAnimationData]   // パーツごとに1つ
       ├─ position_x_array: [{ index: 0, value: 10.0 }, { index: 30, value: 50.0 }]
       ├─ rotation_z_array: [{ index: 0, value: 0.0 }, { index: 15, value: 90.0 }]
       ├─ alpha_array: [{ index: 0, value: 1.0 }]
       └─ (他の配列はすべて空/null)
```

### 属性タグのマッピング

| ssae タグ | ssab 配列 | 値の型 |
|---------|----------|------|
| `CELL` | `cell_array` | PartAnimationDataCellValue |
| `POSX` | `position_x_array` | PartAnimationDataFloatValue |
| `POSY` | `position_y_array` | PartAnimationDataFloatValue |
| `POSZ` | `position_z_array` | PartAnimationDataFloatValue |
| `ROTX` | `rotation_x_array` | PartAnimationDataFloatValue |
| `ROTY` | `rotation_y_array` | PartAnimationDataFloatValue |
| `ROTZ` | `rotation_z_array` | PartAnimationDataFloatValue |
| `SCLX` | `scale_x_array` | PartAnimationDataFloatValue |
| `SCLY` | `scale_y_array` | PartAnimationDataFloatValue |
| `LSCX` | `local_scale_x_array` | PartAnimationDataFloatValue |
| `LSCY` | `local_scale_y_array` | PartAnimationDataFloatValue |
| `ALPH` | `alpha_array` | PartAnimationDataFloatValue |
| `LALP` | `local_alpha_array` | PartAnimationDataFloatValue |
| `PRIO` | `priority_array` | PartAnimationDataIntValue |
| `FLPH` | `flip_h_array` | PartAnimationDataBoolValue |
| `FLPV` | `flip_v_array` | PartAnimationDataBoolValue |
| `HIDE` | `hide_array` | PartAnimationDataBoolValue |
| `PCOL` | `part_color_array` | PartAnimationDataPartColor |
| `SHDR` | `shader_array` | PartAnimationDataShader |
| `VERT` | `vertex_array` | PartAnimationDataVertex |
| `PVTX` | `pivot_x_array` | PartAnimationDataFloatValue |
| `PVTY` | `pivot_y_array` | PartAnimationDataFloatValue |
| `ANCX` | `anchor_x_array` | PartAnimationDataFloatValue |
| `ANCY` | `anchor_y_array` | PartAnimationDataFloatValue |
| `SIZX` | `size_x_array` | PartAnimationDataFloatValue |
| `SIZY` | `size_y_array` | PartAnimationDataFloatValue |
| `IFLH` | `img_flip_h_array` | PartAnimationDataBoolValue |
| `IFLV` | `img_flip_v_array` | PartAnimationDataBoolValue |
| `UVTX` | `uv_translation_x_array` | PartAnimationDataFloatValue |
| `UVTY` | `uv_translation_y_array` | PartAnimationDataFloatValue |
| `UVRZ` | `uv_rotation_z_array` | PartAnimationDataFloatValue |
| `UVSX` | `uv_scale_x_array` | PartAnimationDataFloatValue |
| `UVSY` | `uv_scale_y_array` | PartAnimationDataFloatValue |
| `BNDR` | `bounding_radius_array` | PartAnimationDataFloatValue |
| `MASK` | `mask_array` | PartAnimationDataFloatValue |
| `VERT` (skew) | `skew_array` | PartAnimationDataVec2 | *注: ssae の `VERT` タグは単一の `SsAttributeVertexKeyValue` に頂点オフセットと skew データの両方を含みます。ssab ではこれらが `vertex_array` と `skew_array` に分離されます。* |
| `DEFM` | `deform_array` | PartAnimationDataDeform |
| `TCHG` | `texture_array` | PartAnimationDataUIntValue | *値は `fnv1a(TextureName)` で、`external_textures` から解決可能。空の `TextureName`（作画上の「上書き無し」）はハッシュ化せず `0` に正規化するため、実テクスチャと区別でき `texture` の既定値とも一致する。ステップ評価であり補間されない。* |

### イベント属性（特殊な処理）

イベント型の属性は `PartAnimationData` には格納されません。代わりに、フレーム単位で整理された `EventsPerFrame` に格納されます:

| ssae タグ | ssab イベントテーブル | 備考 |
|---------|-------------------|------|
| `USER` | `EventUserData` | ユーザー定義データ |
| `SIGN` | `EventSignal` | シグナルコマンド |
| `ADIO` | `EventAudio` | オーディオトリガー |
| `IPRM` | `EventInstance` | インスタンス制御 |
| `EFCT` | `EventEffect` | エフェクト制御 |

```
ssab イベント構造:
AnimationData.events: [EventsPerFrame]
  └─ frame_index: 5     (key)
       ├─ users:     [{ part_index: 2, value: ... }]
       ├─ signals:   [{ part_index: 0, value: ... }]
       ├─ audios:    []
       ├─ instances: [{ part_index: 3, value: ... }]
       └─ effects:   []
```

---

## キーフレームデータ

### 補間

| ssab フィールド | ssae フィールド | 備考 |
|---------------|---------------|------|
| `index` (ushort, key) | `time` (i32) | フレームインデックス / 時間 |
| `interpolation` | `ip_type` | 補間タイプの列挙型 |
| `value` | `value` | 属性値 |

ssae ではキーフレームに Bezier/Hermite 補間用の `curve` データ (`SsCurve`) を持つことができ、`start_time`, `start_value`, `end_time`, `end_value`, `sync_start_end` を含みます。ssab もこのうち 4 つの制御値をキーフレームごとに `curve: SsCurve` として保持します（`easing_rate` も同居）。保持するのは必要な 7 テーブル — `PartAnimationDataFloatValue`, `PartAnimationDataIntValue`, `PartAnimationDataPartColor`, `PartAnimationDataShader`, `PartAnimationDataVertex`, `PartAnimationDataVec2`, `PartAnimationDataDeform` — だけです。`sync_start_end` は作画側の利便フラグなので引き継ぎません。残りのキーフレームテーブル（`Cell`, `UInt`, `Bool`, `Instance`, `Effect`）に curve フィールドは無く、補間タイプの列挙型のみでイージング動作が決まります。

### セル属性

| ssab `PartAttributeCell` | ssae `SsAttributeCellKeyValue` | 備考 |
|-------------------------|-------------------------------|------|
| `cellmap_name_hash` (uint32) | `map_id` (i32) | セルマップ名（`.ssce` を除いたファイル名）の fnv1a ハッシュ。コンバート時に `sspj.cellmap_names[map_id]` から解決するため、ssab に一部のセルマップしか同梱されていない場合でもランタイムがハッシュで参照できる。 |
| `name_hash` (uint32) | `name` (String) | 文字列の代わりにハッシュ |

### オーディオ属性

| ssab `PartAttributeAudio` | ssae `SsAttributeAudioKeyValue` | 備考 |
|--------------------------|--------------------------------|------|
| `sound_list_name_hash` (uint32) | `sound_list_id` (i32) | サウンドリスト名から生成したハッシュ |
| `sound_name_hash` (uint32) | `sound_name` (String) | エイリアス名から生成したハッシュ |
| `sound_name` (string) | — | 診断用のエイリアス名（オプション） |
| `loop_num` (int) | `loop_num` (i32) | ループ回数 |

---

## 初期パーツ属性

`InitialPartAttributes` テーブルは各パーツのデフォルト/セットアップ状態を格納します。これは ssae の「セットアップ」アニメーション（`is_setup: true` のアニメーション）に対応します。

### `update_flag` (PartAttributesUpdateFlags)

64ビットのビットマスクで、デフォルト値以外に設定された属性を示します。これにより、ランタイムは初期化時に変更されていない属性をスキップできます。

```
Bit  0: AttributeCell
Bit  1: AttributePosX
Bit  2: AttributePosY
...
Bit 36: AttributeDeform
Bit 37: AttributeTexture
```

合計 38 個のフラグが定義されており、ビット割り当ては `PartAttributesUpdateFlags` の宣言順と一致します。

### デフォルト値

| フィールド | デフォルト |
|---------|---------|
| `position_x/y/z` | 0.0 |
| `rotation_x/y/z` | 0.0 |
| `scale_x/y` | 1.0 |
| `local_scale_x/y` | 1.0 |
| `alpha` | 1.0 |
| `local_alpha` | 1.0 |
| `priority` | 0.0 |
| `flip_h/v` | false |
| `hide` | false |
| `pivot_x/y` | 0.0 |
| `anchor_x/y` | 0.0 |
| `size_x/y` | 1.0 |
| `img_flip_h/v` | false |
| `uv_translation_x/y` | 0.0 |
| `uv_rotation_z` | 0.0 |
| `uv_scale_x/y` | 1.0 |
| `bounding_radius` | 0.0 |
| `mask` | 0.0 |
| `skew_x/y` | 0.0 |
| `texture` | 0 |

---

## エフェクトデータ

### `Effect` (ssab) vs `SsEffectFile` (ssee)

| ssab フィールド | 型 | ssee フィールド | 備考 |
|---------------|------|---------------|------|
| `name_hash` | uint32 (key) | — | 高速検索用ハッシュ |
| `name` | string | `name` | エフェクト名 |
| `fps` | short | `effect_data.fps` | フレームレート |
| `is_lock_rand_seed` | bool | `effect_data.is_lock_rand_seed` | ランダムシード固定 |
| `lock_rand_seed` | short | `effect_data.lock_rand_seed` | シード値 |
| `layout_scale` | Vec2 | `effect_data.layout_scale_x/y` | ssee は個別の int フィールド（パーセント）、ssab は Vec2 |
| `effect_node` | [EffectNode] | `effect_data.node_list` | エフェクトノードツリー |

### `EffectNode` (ssab) vs `SsEffectNode` (ssee)

| ssab フィールド | ssee フィールド | 備考 |
|---------------|---------------|------|
| `index` | `array_index` | ノードインデックス |
| `parent_index` | `parent_index` | 親ノードインデックス |
| `effect_node_type` | `node_type` | Root / Emitter / Particle (`EffectNodeType` 列挙型) |
| `cell_name_hash` | `behavior.cell_name` | セル名の fnv1a ハッシュ |
| `cell_map_name_hash` | `behavior.cell_map_name` | セルマップ名（`.ssce` を除く）の fnv1a ハッシュ |
| `blend_type` | `behavior.blend_type` | `RenderBlendType` 列挙型 — `Mix` / `Add` のみ (パーツで使う `BlendType` フルセットとは別) |
| `behavior` | `behavior.list` | パーティクルの振る舞い（`EffectNodeBehaviorWrapper` でラップされた union 配列） |

### エフェクトノードの振る舞い

ssee ではタグ付き列挙型 (`SsEffectFunctionType`) と名前付きバリアントを使用。ssab では FlatBuffers の union (`EffectNodeBehavior`) を使用:

| ssee タグ名 | ssab Union バリアント | 共有テーブル |
|-----------|---------------------|-----------|
| `"Basic"` | `EffectParticleElementBasic` | 専用テーブル |
| `"OverWriteSeed"` | `EffectParticleElementRndSeedChange` | IntValueEntry |
| `"Delay"` | `EffectParticleElementDelay` | IntValueEntry |
| `"Gravity"` | `EffectParticleElementGravity` | Vec2ValueEntry |
| `"init_position"` | `EffectParticleElementPosition` | RectValueEntry |
| `"init_rotation"` | `EffectParticleElementRotation` | 専用テーブル |
| `"trans_rotation"` | `EffectParticleElementRotationTrans` | 専用テーブル |
| `"trans_speed"` | `EffectParticleElementTransSpeed` | Vec2ValueEntry |
| `"add_tangentiala"` | `EffectParticleElementTangentialAcceleration` | Vec2ValueEntry |
| `"init_vertexcolor"` | `EffectParticleElementInitColor` | 専用テーブル |
| `"trans_vertexcolor"` | `EffectParticleElementTransColor` | 専用テーブル |
| `"trans_colorfade"` | `EffectParticleElementAlphaFade` | Vec2ValueEntry |
| `"init_size"` | `EffectParticleElementSize` | RectAndVec2ValueTable |
| `"trans_size"` | `EffectParticleElementTransSize` | RectAndVec2ValueTable |
| `"add_pointgravity"` | `EffectParticlePointGravity` | 専用テーブル |
| `"TurnToDirection"` | `EffectParticleTurnToDirectionEnabled` | FloatValueEntry |
| `"InfiniteEmit"` | `EffectParticleInfiniteEmitEnabled` | IntValueEntry |

### 引き継がれないフィールド

- `SsEffectNode.name` (ノード名、エディタ用)
- `SsEffectNode.visible` (エディタの表示フラグ)
- `SsEffectModel.bgcolor` (エディタの背景色)

### VarianceValue のフラット化

ssee では多くのエフェクトパラメータが `VarianceValue<T>`（`value` と `subvalue` の最小/最大範囲）を使用します。ssab ではこれらがフラット化されます:

```
ssee: speed: VarianceValue<f32> { value: 5.0, subvalue: 10.0 }
ssab: speed_min_value: 5.0, speed_max_value: 10.0
```

---

## サウンドデータ

### `SoundList` (ssab) vs `SsAudioPack` (ssse)

| ssab フィールド | 型 | ssse フィールド | 備考 |
|---------------|------|---------------|------|
| `name_hash` | uint32 (key) | — | 高速検索用ハッシュ |
| `name` | string | `name` | サウンドリスト名 |
| `table_data` | [SoundFile] | `table.sounds` | サウンドファイルエントリ |

### `SoundFile` (ssab) vs `SsSoundFile` (ssse)

| ssab フィールド | ssse フィールド | 備考 |
|---------------|---------------|------|
| `name_hash` | — | 高速検索用ハッシュ (Alias名から生成) |
| `name` | `alias` | サウンドエイリアス名 |
| `file_path` | `filename` | ファイルパス |
| `file_path_hash` | — | ファイルパスから生成したハッシュ |
| `time_total` | `msec` | 再生時間（ミリ秒） |


---

## ビットマップフォントデータ

### `FontBitmap` (ssab) vs `SsCharMap` (fnt)

| ssab フィールド | fnt フィールド | 備考 |
|---------------|------------|------|
| `name_hash` | — | 高速検索用ハッシュ |
| `name` | — | フォント名 |
| `name_face` | `info.face` | フォントフェイス名 |
| `size` | — | フォントサイズ |
| `padding_l/r/u/d` | — | パディング値 |
| `base` | `common.base` | ベースライン |
| `height_line` | `common.line_height` | 行の高さ |
| `scale` | `common.scale_w/h` | テクスチャスケール |
| `spacing` | `info.spacing` | グリフのスペーシング (x, y) |
| `size_texture_original` | — | 元のテクスチャサイズ |
| `name_texture` | `pages[*].file` | テクスチャファイル名 |
| `name_texture_hash` | — | テクスチャファイル名のハッシュ配列 |
| `table_data` | `chars` | グリフデータ |

### `FontGlyphBitmap` (ssab) vs `SsCharMapChar` (fnt)

| ssab フィールド | fnt フィールド | 備考 |
|---------------|------------|------|
| `id` | `id` | 文字 ID（Unicode コードポイント） |
| `atlas` | `src_rect` | アトラス内のソース矩形 |
| `offset` | `offset` | グリフ描画オフセット |
| `width_advance` | `xadvance` | 水平方向の送り幅 |
| `text` | — | 文字テキスト（fnt では id から導出） |
| `page` | `page` | テクスチャページインデックス |
| `channel` | — | チャンネルインデックス |

---

## SSQB: シーケンスバイナリ

**FlatBuffers 識別子:** `"SSQB"`

### `SsSequenceBinary` (ssqb) vs `SsSequencePack` (ssqe)

| ssqb フィールド | ssqe フィールド | 備考 |
|---------------|---------------|------|
| `version` | `version` | フォーマットバージョン |
| `sequence_list` | `sequence_list` | シーケンス定義 |

### `Sequence` (ssqb) vs `SsSequence` (ssqe)

| ssqb フィールド | ssqe フィールド | 備考 |
|---------------|---------------|------|
| `name` | `name` | シーケンス名 |
| `name_hash` | — | 高速検索用ハッシュ |
| `index` | `index` | シーケンスインデックス |
| `type` | `sequence_type` | Last / Keep / Top |
| `list` | `list` | シーケンス項目 |

### `SequenceItem` (ssqb) vs `SsSequenceItem` (ssqe)

| ssqb フィールド | ssqe フィールド | 備考 |
|---------------|---------------|------|
| `ref_anime_pack` | `ref_anime_pack` | 参照アニメーションパック名 |
| `ref_anime_pack_hash` | — | 高速検索用ハッシュ |
| `ref_anime` | `ref_anime` | 参照アニメーション名 |
| `ref_anime_hash` | — | 高速検索用ハッシュ |
| `repeat_count` | `repeat_count` | 繰り返し回数 |

シーケンスフォーマットは ssqe と ssqb 間でほぼ同一であり、最小限の変換のみが行われます。

---

## 列挙型のマッピング

すべての列挙型は ss*e と ssab 間で 1:1 の対応があります。主な列挙型:

### BlendType
`Mix`, `Mul`, `Add`, `Sub`, `Mulalpha`, `Screen`, `Exclusion`, `Invert`, `Mul2`, `Div2`, `Screen2`, `Overlay2`

### InterpolationType
`None`, `Linear`, `Hermite`, `Bezier`, `Acceleration`, `Deceleration`, `EaseIn`, `EaseOut`, `EaseInOut`, `EaseExponentialIn/Out/InOut`, `EaseSineIn/Out/InOut`, `EaseElasticIn/Out/InOut`, `EaseBounceIn/Out/InOut`, `EaseBackIn/Out/InOut`

### BoundsType
`None`, `Quad`, `Aabb`, `Circle`, `CircleSmin`, `CircleSmax`

### TexWrapMode
`Clamp`, `Repeat`, `Mirror`

### TexFilterMode
`Nearest`, `Linear`

---

## プリミティブ型

| ssab 型 | 定義 | ss*e 対応 |
|--------|------|---------|
| `Vec2` | `{ v1: float, v2: float }` | `SsPoint2 { x, y }` |
| `Rect` | `{ x1, y1, x2, y2: float }` | `SsIRect { x1, y1, x2, y2 }` または pos+size ペア |
| `U8Rect` | `{ a, r, g, b: ubyte }` | `SsU8Color { a, r, g, b }` |

---

## ssab の主要な設計方針

1. **ssae 単位の変換**: 各 ssab は単一の ssae アニメーションパックに対応し、そのパックが実際に使用する ss*e リソース (ssce, ssee, ssse, fnt) のみをバンドルします。sspj プロジェクトファイルは ssab に含まれません。

2. **ハッシュベースの検索**: すべての名前付きエンティティに `name_hash: uint32` フィールドを含み、ランタイムでの文字列比較を回避して O(1) 検索を実現。

3. **カラム型アニメーションデータ**: 属性値はタグ付き列挙型ではなく型付き配列として格納され、フレーム間の単一属性に対する効率的なイテレーションが可能。

4. **イベントの分離**: イベント型属性（ユーザーデータ、シグナル、オーディオ、インスタンス、エフェクト）はパーツごとではなくフレームごとに格納され、フレームベースのイベントディスパッチを最適化。

5. **フラットなパーツ階層**: パーツは `parent_index` 参照付きのフラット配列として格納（ssae と同様）。ただしエディタ専用フィールドは除去。

6. **Union ベースの型判別**: FlatBuffers の union が ssae のタグ文字列ディスパッチを置き換え、ランタイムでの文字列マッチングなしに型安全なアクセスを提供。

7. **ゼロコピー読み込み**: デフォルト（ランタイム）モードでは FlatBuffers のゼロコピーデシリアライゼーションを使用。`converter` feature を有効にすると Object API による変更が可能。

8. **文字列からハッシュへの変換**: エンティティ間の参照（インスタンス参照、セル名、ラベルなど）は文字列の代わりに `uint32` ハッシュを使用。デバッグ目的で元の文字列がハッシュと共に保持される場合もある。
