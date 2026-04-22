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
| `texture` | `texture_array` | テクスチャ変更（上書き）用 ID。 |

---

## 複雑な属性のマッピング（プーリング）

複雑なデータ構造の重複を避けるため、複雑な属性は `FrameData` 内のプールに格納されます。これらは `PartState` 内で `short` 型のインデックスとして参照されます。

| PartState 参照フィールド | ソース: SSAB `PartAnimationData` 配列 | FrameData 内のテーブル | 説明 |
|:---|:---|:---|:---|
| `cell` | `cell_array` | `cells` | セルマップとセル名ハッシュへの参照。通常描画に使用されるテクスチャはここから取得されます。 |
| `part_color` | `part_color_array` | `parts_color` | 補間済みの 4頂点カラーとブレンドタイプ。 |
| `shader` | `shader_array` | `shaders` | 補間済みのシェーダーパラメータとマップ ID。 |
| `vertex` | `vertex_array` | `vertices` | 補間済みの頂点座標オフセット。 |
| `deform` | `deform_array` | `deforms` | 補間済みのメッシュ変形座標。 |

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
