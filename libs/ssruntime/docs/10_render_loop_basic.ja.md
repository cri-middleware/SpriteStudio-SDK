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
- **座標 (position_x, position_y)**: 位置。
- **Z 軸回転 (rotation_z)**: 回転。
- **スケール (scale_x, scale_y)**: 拡大縮小。
- **不透明度 (alpha)**: 透明度。
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
int current_frame = ss_runtime_update(runtime_ctx, delta_ms);

// 更新後に描画関数を呼び出す
Render(runtime_ctx);
```

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

- **シーングラフ / 階層ノード方式**: 
  パーツの数だけエンジンのオブジェクトを生成し、`ssab` メタデータの `parent_index` を元にエンジンの Transform 階層を構築します。`FrameData` から取得した各 `PartState` のローカル値を各オブジェクトの `transform` に反映させることで、最終的な描画位置（ワールド座標）の計算をエンジンに委ねることができます。実装は容易ですが、パーツ数に比例してエンジン側のノード管理オーバーヘッドが増大するため、注意が必要です。

---
[目次に戻る](./README.ja.md)
