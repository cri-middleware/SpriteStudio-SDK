# 実装フェーズ 3：基本的な描画の実装 (Basic Rendering)

アニメーションの最初の一歩として、まずは **フレーム 0 を正しく描画（静止画表示）** することを目指します。

## 1. 最初のステップ：フレーム 0 の描画
「キャラクターアニメーション テンプレート」などの単純なデータを表示し、正しく座標とテクスチャが反映されるかを確認します。

1. `ss_runtime_update(runtime_ctx, 0.0f)` を呼び出し、再生時間を進めずに初期状態にします。
2. `ss_runtime_get_part_states(runtime_ctx, &parts, &count)` を呼び出し、現在のフレームの全パーツの状態を取得します。
3. パーツのリスト（`parts`）をループして、基本的なアトリビュートを適用して描画します。

```cpp
// 基本的な描画ループの例 (C++)
int part_count = 0;
const ss_part_state* part_states = nullptr;

// 現在のフレームにおける全パーツの計算済み状態を取得
ss_runtime_get_part_states(runtime_ctx, &part_states, &part_count);

for (int i = 0; i < part_count; i++) {
    const ss_part_state& state = part_states[i];

    // 非表示フラグの確認
    if (state.hide) continue;

    // 座標・回転・スケールの取得
    // これらは libssruntime 内部で親子関係が計算された後の「ワールド座標」相当の値です
    float x = state.position_x;
    float y = state.position_y;
    float rotation = state.rotation_z;
    float scale_x = state.scale_x;
    float scale_y = state.scale_y;

    // テクスチャと UV の取得
    // state.cell_index を元に、初期化時に構築したキャッシュからテクスチャと UV を引く
    Texture* tex = GetTextureByCellIndex(state.cell_index);
    Rect uv = GetUVByCellIndex(state.cell_index);

    // 描画の実行
    DrawTexture(tex, x, y, rotation, scale_x, scale_y, uv, state.alpha);
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

- **階層ノード方式 (Godot Node2D / Unity GameObject)**: 
  パーツの数だけエンジンのオブジェクトを生成します。各 `PartState` の値をそのまま各オブジェクトの `transform` に反映させます。libssruntime が親子関係を計算済みであるため、エンジン側で親子階層を組まずに、すべてのノードをフラットな子として管理する（ワールド座標として扱う）のが最もシンプルです。
  
- **フラットレンダリング方式 (CanvasItem / Custom Shader)**:
  1 つの描画関数内でループを回し、連続して描画命令を発行します。Godot の `_draw` 関数内でのループ処理や、頂点バッファへの流し込みがこれに当たります。パフォーマンス面で優れています。
