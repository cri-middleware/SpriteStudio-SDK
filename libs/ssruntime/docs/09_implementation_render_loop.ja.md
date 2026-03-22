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

// FlatBuffers でパース
auto frame_data = ss::runtime::GetFrameData(frame_buf);
```

## 3. レンダリングの実行
`frame_data` に含まれる各パーツの状態をループで処理し、描画を行います。

```cpp
if (frame_data->parts()) {
    for (auto part_state : *frame_data->parts()) {
        // パーツのインデックスを取得
        int part_idx = part_state->part_index();
        
        // アトリビュート（ローカル座標系）の取得
        float px = part_state->position_x();
        float py = part_state->position_y();
        float rz = part_state->rotation_z();
        float sx = part_state->scale_x();
        float sy = part_state->scale_y();
        float alpha = part_state->alpha();

        // 非表示フラグの確認
        if (part_state->hide()) continue;

        // 1. ワールド行列の計算
        // フェーズ 1 で構築した親子関係を使い、親の行列を掛け合わせて
        // このパーツの最終的なワールド行列を計算してください。
        Matrix4x4 world_matrix = CalculateWorldMatrix(part_idx, px, py, rz, sx, sy);

        // 2. セル情報の取得
        if (part_state->update_flag() & ss::runtime::UpdateAttributeFlags_AttributeCell) {
            int cell_idx = part_state->cell();
            // cell_idx を元に、テクスチャ内での UV 座標を算出
            Rect uv_rect = GetUVFromCellIndex(cell_idx);
            
            // 3. 描画
            // 計算した行列、UV、不透明度、合成方法（BlendType）などを用いて描画を実行
            DrawPart(world_matrix, uv_rect, alpha);
        }
    }
}
```

## 4. イベントの処理
ユーザーデータなどのイベントがこのフレームに存在する場合、それを取得して処理します。

```cpp
// ※注: ssruntime の内部実装や ssab のイベント構造に基づき、
// アニメーションデータから現在のフレームに関連するイベントを抽出してください。
// 今後の SDK アップデートにより、イベント取得用の便利な API が提供される予定です。
```

## 5. 補足：行列計算の注意点
- **座標系**: SpriteStudio は基本的に左上原点ですが、ゲームエンジンによっては中心原点や左下原点の場合があります。必要に応じて Y 座標の反転やオフセットの調整を行ってください。
- **回転の向き**: 回転角（RZ等）の正負の向きもエンジンに合わせて調整が必要な場合があります。
- **不透明度の継承**: 親パーツの不透明度が子に影響するかどうかは、パーツの設定やエンジンの仕様に合わせて実装してください。
