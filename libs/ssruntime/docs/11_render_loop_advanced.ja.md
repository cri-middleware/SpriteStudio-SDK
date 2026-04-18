# 実装フェーズ 3：更新・描画 (Render Loop)

毎フレーム実行される、アニメーションの更新とレンダリングのフェーズです。

## 1. アニメーションの更新
経過時間（デルタタイム）を渡してアニメーションを進め、現在のフレーム番号を取得します。

```cpp
// delta は前回の更新からの経過秒数 (e.g. 1/60)
int current_frame = ss_runtime_update(context, delta);
```

## 2. フレームデータの取得
指定したフレームの描画に必要な情報を FlatBuffers 形式で取得します。

```cpp
uint8_t* frame_buf = nullptr;
size_t frame_len = 0;
// 現在のフレーム番号（または指定フレーム）のデータを取得
ss_runtime_get_frame_data(context, current_frame, &frame_buf, &frame_len);

if (frame_buf == nullptr) return; // データがない場合はスキップ

// FlatBuffers でパース
auto frame_data = ss::runtime::GetFrameData(frame_buf);
```

## 3. レンダリングの実行
`frame_data` に含まれる各パーツの状態（`PartState`）をループで処理し、描画を行います。

```cpp
if (frame_data->parts()) {
    for (auto part_state : *frame_data->parts()) {
        // 1. 非表示フラグの確認
        if (part_state->hide()) continue;

        // 2. 基本アトリビュートの取得
        // position, rotation, scale は親子関係を考慮した計算（後述）が必要
        float px = part_state->position_x();
        float py = part_state->position_y();
        float pz = part_state->position_z(); // Z座標（優先度とは別）
        
        float rx = part_state->rotation_x();
        float ry = part_state->rotation_y();
        float rz = part_state->rotation_z();
        
        float sx = part_state->scale_x();
        float sy = part_state->scale_y();

        // 不透明度
        float alpha = part_state->alpha();
        float local_alpha = part_state->local_alpha(); // パーツ固有の不透明度

        // 3. セル情報の取得
        // PartState->cell() は frame_data->cells() へのインデックスです
        int cell_attr_idx = part_state->cell();
        if (cell_attr_idx >= 0 && frame_data->cells()) {
            auto cell_attr = frame_data->cells()->Get(cell_attr_idx);
            
            // cell_attr->map_id(): 使用する CellMap のインデックス
            // cell_attr->name_hash(): セルの名前のハッシュ
            
            // これらを元に、初期化時に構築したマップから UV やテクスチャを取得します
            Texture* tex = GetTexture(cell_attr->map_id());
            Rect uv_rect = GetUVByHash(cell_attr->map_id(), cell_attr->name_hash());

            // 4. 描画の実行
            // ワールド行列を計算し、テクスチャ、UV、不透明度を渡して描画
            Matrix4x4 world_matrix = CalculateWorldMatrix(part_state->part_index(), ...);
            DrawPart(world_matrix, tex, uv_rect, alpha * local_alpha);
        }
    }
}
```

## 4. イベントの処理
ユーザーデータやシグナルなどのイベントは、アニメーションデータ（`SsAnimeBinary`）から直接取得する必要があります。
`libssruntime` の `Context` を通じて現在の再生フレームに基づいたイベントを抽出する実装を Player 側で行ってください。

## 5. 行列計算の注意点
- **継承関係**: `PartState` に格納されている値は、親子関係が計算された後の値（ワールド座標相当）ではなく、**そのフレームにおける各アトリビュートの最終的な値**です。ただし、SpriteStudio の仕様上、行列として親の結果を継承する必要があるため、計算順序には注意してください。
- **Flip**: `flip_h()`, `flip_v()`（パーツ全体の反転）と `img_flip_h()`, `img_flip_v()`（テクスチャのみの反転）を正しく反映させてください。
- **Skew**: `skew_x()`, `skew_y()` による歪み変形に対応する場合、行列計算に歪みの要素を追加します。

---
[目次に戻る](./README.ja.md)
