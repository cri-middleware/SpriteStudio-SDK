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

1. **ワールド行列の計算を有効にする**: `ss_runtime_set_world_matrix_calculation_enabled(ctx, true)` を呼び出します（デフォルトで有効です）。
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
