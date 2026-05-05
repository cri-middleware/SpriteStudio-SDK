# 実装フェーズ 3：基本的な描画の実装 (Basic Rendering)

アニメーションの最初の一歩として、まずは **フレーム 0 を正しく描画（静止画表示）** することを目指します。

## 1. 最初のステップ：フレーム 0 の描画
「キャラクターアニメーション テンプレート」などの単純なデータを表示し、正しく座標とテクスチャが反映されるかを確認します。

1. `ss_runtime_update` 等の関数を呼び出し、再生時間を進めずに初期状態の `FrameData` （FlatBuffersチャンク）を取得します。
2. 取得したチャンクを Player 側で `FrameData` テーブルとしてデコードし、パーツのリスト（`parts`）を取得します。
3. パーツのリストをループして、基本的なアトリビュートを適用して描画します。

```cpp
// 基本的な描画ループの概念を示す擬似コード (C++ / FlatBuffers)

// libssruntime から現在のフレームの計算済み状態（FlatBuffers のチャンク）を取得
const uint8_t* chunk_data = nullptr;
size_t chunk_size = 0;
ss_runtime_get_frame_data(runtime_ctx, &chunk_data, &chunk_size);

// FlatBuffers のデコード
auto frame_data = GetFrameData(chunk_data);
auto parts = frame_data->parts();

for (size_t i = 0; i < parts->size(); i++) {
    auto part = parts->Get(i);

    // 非表示フラグの確認
    if (part->hide()) continue;

    // 座標・回転・スケールの取得
    // これらは libssruntime から返される「ローカル」の値です。
    // 親子関係を反映させるには、Player側でTransform階層を組むか、行列演算を行う必要があります。
    float x = part->position_x();
    float y = part->position_y();
    float rotation = part->rotation_z();
    float scale_x = part->scale_x();
    float scale_y = part->scale_y();

    // テクスチャと UV の取得
    // cell_index を元に、初期化時に構築したキャッシュからテクスチャと UV を引く
    Texture* tex = GetTextureByCellIndex(part->cell_index());
    Rect uv = GetUVByCellIndex(part->cell_index());

    // 描画の実行
    DrawTexture(tex, x, y, rotation, scale_x, scale_y, uv, part->alpha());
}
```

## 2. 最低限実装が必要なアトリビュート
まずは以下の項目を反映することから始めます。これが出来れば、多くのアニメーションが形になります。

- **セル (cell_index)**: libssruntime が返すインデックスを使い、事前にロードしたテクスチャや UV を特定します。
- **座標 (position_x, position_y)**: ピクセル単位（または設定した単位）の座標。
- **Z 軸回転 (rotation_z)**: 度数法（Degrees）による回転角度（0〜360）。
- **スケール (scale_x, scale_y)**: 等倍を 1.0 とした倍率。
- **不透明度 (alpha)**: 0.0（完全透明）〜 1.0（完全不透明）の範囲の正規化された値。
- **ピボット (pivot_x, pivot_y)**: セルの中心を (0.0, 0.0)、端を ±0.5 とした正規化座標。
- **非表示 (hide)**: 非表示フラグが立っている場合は描画をスキップ。

## 3. 座標変換とエンジンの合わせ込み
SpriteStudio とゲームエンジンの座標系の違いを吸収する必要があります。

- **原点**: SpriteStudio は「左上原点」ですが、libssruntime が返す座標は通常、アニメーションの中心（Pivot）を基準とした相対座標です。
- **軸の向き**: Godot などのエンジンでは Y 軸の下向きが正（Y-Down）ですが、他のエンジンでは上向きが正（Y-Up）の場合があります。
- **行列計算**: libssruntime は親子関係を考慮した最終的な各パーツの座標・回転・スケールを個別に返します。Player 側ではこれらを組み合わせてワールド行列を生成するか、エンジンのトランスフォーム機能に直接セットします。

> **【AI 実装者向け制約 / Implementation Directives for AI】**
> libssruntime から出力される（FlatBuffers 経由で取得する）座標系は `X-Right, Y-Down` を基準としています。
> AI アシスタントがコードを生成する際は、ターゲットエンジンに応じた座標変換を**必ず以下のルールに従って**実装に組み込むこと。
> - **Unity (Y-Up / 左手系) の場合:** 取得した `position_y` に `-1` を乗算し、`rotation_z` に `-1` を乗算して Y-Up に変換せよ。
> - **Godot 2D (Y-Down / 右手系) の場合:** 座標変換は不要。取得した値をそのまま代入せよ。
> - **UE5 (Z-Up / 左手系) の場合:** `Y` と `Z` の軸を入れ替え、さらに適切な反転を行って X-Forward, Y-Right, Z-Up に適応させよ。

```cpp
// Godot (C++) での描画適用例
Transform2D t;
t.set_origin(Vector2(state.position_x, state.position_y));
t.set_rotation(Math::deg_to_rad(state.rotation_z));
t.set_scale(Vector2(state.scale_x, state.scale_y));

// Godot の CanvasItem API を使用
draw_set_transform_matrix(t);
draw_texture_rect_region(texture, rect, uv_rect, Color(1, 1, 1, state.alpha));
```

## 4. 合成モード (Blend Mode) の適用
パーツごとに指定された合成モード（`state.blend_type`）を反映します。

- **Mix (通常)**: アルファブレンディング
- **Add (加算)**: 加算合成
- **Sub (減算)**: 減算合成
- **Mul (乗算)**: 乗算合成

## 5. フレーム番号の取得とループ
静止画ができたら、毎フレーム時間を進めてアニメーションさせます。

```cpp
// 毎フレームの更新ループ
float delta_ms = GetDeltaTime() * 1000.0f; // ミリ秒で渡す
float current_frame = ss_runtime_update(runtime_ctx, delta_ms);

// 更新後に描画関数を呼び出す
Render(runtime_ctx);
```

> **注意**: `ss_runtime_update` は現在、浮動小数 (`float`) のフレーム番号を返します。整数フレームとして扱いたい場合は、戻り値を `floor` するか、前述のサブフレーム設定を Player 側に実装して制御してください。

## 6. この段階での動作確認 (Validation)
基本的な行列計算と描画パイプラインが正しいか、`overall.sspj` の以下のデータを使用して確認します。

- **確認対象**: `Basic.ssae`
    - **チェック内容**: 
        - パーツが正しい位置に出現し、親子関係に沿って動いているか。
        - スケールや回転が親の動きを正しく継承しているか。
- **確認対象**: `BlendMode.ssae`
    - **チェック内容**: 
        - 各パーツに指定された合成モード（Mix, Add, Sub, Mul）が視覚的に正しく反映されているか。

# 描画方式別の実装パターン（詳細）

- **ダイレクト描画 / バッチレンダリング方式**:
  1 つの描画関数内でループを回し、連続して描画命令を発行します。この方式は、近年のデータドリブンなアーキテクチャと非常に相性が良く、高いパフォーマンスを発揮します。`ssab` の親子構造を辿り、自前で行列演算を行ってワールド座標を算出して頂点バッファへ流し込む実装が必要ですが、多くのケースでこのアプローチが有利になります。

### 効率的な行列演算の手順
`libssruntime` から取得できる `FrameData` 内のパーツリストは、**「親パーツが必ず子パーツよりも前に出現する」**ように並んでいます。これを利用すると、非常に効率的に各パーツのワールド行列を計算できます。

また、SpriteStudio 独自の継承ルール（アンカーポイントやローカルスケールの扱いなど）を正しく計算した「ワールド行列」を一括で取得する機能が `libssruntime` に備わっています。自前で行列演算を実装する代わりに、SDK が提供する **`ss_runtime_get_world_matrices`** を利用することを強く推奨します。

1. **描画モードを Direct に設定する**: `ss_runtime_set_render_mode(ctx, 0)` を呼び出します（`0` = Direct、デフォルト）。シーングラフ方式を使う場合は `1`（SceneGraph）を指定し、ワールド行列の事前計算をスキップして `FrameData` の各パーツのローカル属性から自前で transform を組み立てます。
2. **ワールド行列バッファを取得する**: `ss_runtime_get_world_matrices` を呼び出し、現在のフレームの全パーツ分の行列が格納された配列へのポインタと要素数を取得します。
3. **描画ループで利用する**:
   - `FrameData` 内のパーツリストをループします。
   - `world_matrices[i]` には、そのパーツの「描画用行列 (Drawing Matrix)」が既に計算された状態で格納されています。

```cpp
// ss_runtime_get_world_matrices を利用した効率的な描画ループの例 (C++)
const float* world_matrices = nullptr;
size_t matrix_count = 0;
ss_runtime_get_world_matrices(runtime_ctx, &world_matrices, &matrix_count);

auto frame_data = GetFrameData(chunk_data);
auto parts = frame_data->parts();

for (size_t i = 0; i < parts->size(); i++) {
    auto part = parts->Get(i);

    // 描画の実行 (非表示パーツでなければ)
    if (!part->hide()) {
        // i 番目の行列（16要素の float 配列）をそのままエンジンに渡す
        const float* m = &world_matrices[i * 16];
        DrawPart(m, part->cell_index(), part->alpha());
    }
}
```

- **描画用行列 (drawing matrix)**: 現在のパーツのローカルスケール等、子に継承「させない」パラメータを含んだ、描画にそのまま使える行列です。`libssruntime` 内部では、子パーツへの継承には別途「継承用行列 (Inheritance Matrix)」が使用されていますが、外部へ公開されるのは常にこの描画用行列です。

この機能を使用することで、SpriteStudio 特有の複雑な行列計算を自前で実装することなく、正しく高速な描画が可能になります。

### Cell メタデータと UV の事前解決バッファ

`libssruntime` は cellmap からの値解決を Setup 時に一度だけ行い、毎フレームは Player が SoA バッファをインデックスで読むだけで済むよう、以下のバッファを公開しています。Player 側で `cellmap->cells()->LookupByKey(name_hash)` のような毎フレームの二分探索を行う必要はありません。

- **`ss_runtime_get_cell_meta`**: パーツごとの `[pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]` を 6 floats × num_parts のレイアウトで返します。`(rect_left, rect_top)` はテクスチャ・ピクセル空間でのセル矩形の左上原点、`(size_w, size_h)` はセルの幅・高さです。セルが無いパーツのスロットはゼロで埋められます。
- **`ss_runtime_get_cell_texture_hashes`**: パーツごとの「現在セルが属する cellmap」の `name_hash` (FNV-1a, `uint32_t × num_parts`) を返します。値は SSAB の `cellmaps[map_id].name_hash` と一致するため、Player は自前の `name_hash → Texture` テーブルを直接引いて目的のテクスチャを得られます。`FrameData::cells()->Get(...)->map_id()` → `cellmaps()->Get(map_id)->name_hash()` という 2 段の FlatBuffer ルックアップは不要です。セルが無いパーツのスロットはゼロです。
- **`ss_runtime_get_local_uvs`**: パーツごとの 5 頂点分の UV を `[u0, v0, u1, v1, u2, v2, u3, v3, u4, v4]` (テクスチャ・ピクセル空間、10 floats × num_parts) で返します。flip / UV 平行移動 / 回転 / スケールが反映済みの最終 UV です。
- **`ss_runtime_get_shape_vertices`** / **`ss_runtime_get_shape_vertex_box_coords`** / **`ss_runtime_get_shape_vertex_counts`**: Shape パーツの描画用 SoA 群です。
    - `shape_vertices`: ローカル頂点座標 12 頂点 × `(x, y)` = 24 floats × num_parts。pivot オフセットと座標系 (Y-Up / Y-Down) の処理は Brain 側で完了済みなので、Player は `world_matrix * v` をかけて描画するだけです。
    - `shape_vertex_box_coords`: 各頂点の正規化バウンディングボックス座標 `(fx, fy)` 12 頂点 × 2 floats × num_parts。`(0, 0)` が LT 角、`(1, 1)` が RB 角に対応し、pivot や座標系の影響を受けません。Player は part の 4 隅カラー (`pc->lt/rt/lb/rb`) を `(1-fx)*(1-fy), fx*(1-fy), (1-fx)*fy, fx*fy` で重み付け加算するだけで、各頂点色を確定できます。
    - `shape_vertex_counts`: 有効頂点数 `int32 × num_parts`。Shape 以外のパーツでは 0、頂点・box-coord スロットはゼロ埋めです。
- **`ss_runtime_get_z_order`**: パーツの描画順位 (priority と階層位置から決定) を `int32_t × num_parts` で返します。

```cpp
const float* cell_meta = nullptr;
size_t cell_meta_len = 0;
ss_runtime_get_cell_meta(runtime_ctx, &cell_meta, &cell_meta_len);

const uint32_t* cell_texture_hashes = nullptr;
size_t cell_texture_hashes_len = 0;
ss_runtime_get_cell_texture_hashes(runtime_ctx, &cell_texture_hashes, &cell_texture_hashes_len);

const float* local_uvs = nullptr;
size_t local_uvs_len = 0;
ss_runtime_get_local_uvs(runtime_ctx, &local_uvs, &local_uvs_len);

auto frame_data = GetFrameData(chunk_data);
auto parts = frame_data->parts();

for (size_t i = 0; i < parts->size(); i++) {
    auto part = parts->Get(i);
    if (part->hide() || part->cell() < 0) continue;

    // cell_meta の per-part スライス (6 floats)
    const float* meta = cell_meta + i * 6;
    // [pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]

    // local_uvs の per-part スライス (10 floats: 4 corners + center)
    const float* uvs = local_uvs + i * 10;

    // テクスチャ参照は cell_texture_hashes から直接 (毎フレームの FlatBuffer ルックアップ不要)
    uint32_t tex_hash = cell_texture_hashes[i];
    Texture* tex = ResolveTextureByNameHash(tex_hash);

    DrawPart(tex, meta, uvs, /* world matrix, alpha, ... */);
}
```

Player のテクスチャ管理は **Setup 時に `cellmaps[i].name_hash` をキーに自前のテクスチャテーブル** を構築しておくだけで、毎フレームの解決は SoA 配列の 1 アクセスで完結します。`FrameData::cells()` を Player から触る必要はもうありません。

- **シーングラフ / 階層ノード方式**: 
  パーツの数だけエンジンのオブジェクトを生成し、`ssab` メタデータの `parent_index` を元にエンジンの Transform 階層を構築します。`FrameData` から取得した各 `PartState` のローカル値を各オブジェクトの `transform` に反映させることで、最終的な描画位置（ワールド座標）の計算をエンジンに委ねることができます。実装は容易ですが、パーツ数に比例してエンジン側のノード管理オーバーヘッドが増大するため、注意が必要です。

---

# 実装フェーズ 4：パーツ種別とアトリビュートの詳細

SpriteStudio7 には多くのパーツ種別とアトリビュートが存在します。Player 実装の段階に合わせて、対応する項目を増やしていくためのリファレンスです。

## 1. 利用するパーツタイプ

### 基本パーツ
- **PartTypeNull**: 描画を行わないヌルパーツ。親子関係の中継用として使われます。
- **PartTypeNormal**: 通常の矩形パーツ。最も頻繁に使用されます。

### v5〜v6 拡張系パーツ
- **PartTypeInstance**: インスタンスパーツ。別のアニメーションをこのパーツの子として再生します。
- **PartTypeEffect**: エフェクトパーツ。パーティクルエフェクトを表示します。
- **PartTypeMesh**: メッシュパーツ。頂点を変形させて表示します。
- **PartTypeMask**: マスクパーツ。以降の描画にマスクをかけます。

### 7.1 拡張系
- **PartTypeShape**: シェイプパーツ。単純な図形を表示します。
- **PartTypeText**: テキストパーツ。
- **PartTypeNines**: 9スライスパーツ。
- **PartTypeAudio**: 音声再生用のパーツ。
- **PartTypeCamera**: カメラ制御用のパーツ。

### ボーン・関連パーツ
- **PartTypeArmature**, **PartTypeJoint**, **PartTypeMovenode**, **PartTypeConstraint**, **PartTypeBonepoint**, **PartTypeTransformConstraint**

## 2. パーツ操作 API
Player は、データによる指定とは別に、プログラムからパーツの状態を制御する API を提供することを推奨します。

- **setPartVisible(partName, visible)**: 指定したパーツの表示・非表示を強制的に切り替えます。データ上の `AttributeHide` よりも優先して制御したい場合に使用します。

## 3. 利用するアトリビュート

### すべてのパーツ（Null を含む）
- **AttributePosX, AttributePosY, AttributePosZ**: 座標。
- **AttributeRotX, AttributeRotY, AttributeRotZ**: 回転。
- **AttributeSclX, AttributeSclY**: スケール。
- **AttributeLosclX, AttributeLosclY**: ローカルスケール。
- **AttributeAnchorX, AttributeAnchorY**: アンカーポイント（ピボット）。

### 描画系（Normal, Mesh, Nines 等）
- **AttributeCell**: 参照セル。
- **AttributeAlpha, AttributeLoAlpha**: 不透明度。
- **AttributePrio**: 描画優先度（Zオーダー）。
- **AttributeFlipH, AttributeFlipV**: 左右・上下反転。
- **AttributeHide**: 非表示。
- **AttributePivotX, AttributePivotY**: ピボット位置。
- **AttributeSizeX, AttributeSizeY**: サイズ。
- **AttributeImgFlipH, AttributeImgFlipV**: セル（画像）自体の反転。
- **AttributeUvtX, AttributeUvtY, AttributeUvrZ, AttributeUvsX, AttributeUvsY**: UV 座標の移動・回転・スケール。
- **AttributeBoundr**: 当たり判定の半径。

### 特殊系
- **Mask**: `AttributeMask`
- **Effect**: エフェクト開始イベント
- **Instance**: インスタンス再生設定
- **UserData**: ユーザーデータイベント
- **Signal**: シグナルイベント

## 3. この段階での動作確認 (Validation)
主要なアトリビュート（反転、サイズ、ピボット等）が正しく反映されているかを確認します。

- **確認対象**: `Bounding.ssae`
    - **チェック内容**: 
        - 当たり判定用パーツの座標やサイズ（半径）が、正しくアトリビュートとして取得できているか。
        - 反転（Flip）などが親子関係を通じて正しく計算されているか。

---
[目次に戻る](./README.ja.md)
