# 実装フェーズ 4：コンポーネント実装 (Instance / Effect / Mesh)

SpriteStudio の高度な表現を支える「入れ子構造のアニメーション」「パーティクルエフェクト」「メッシュ変形」の実装ガイドです。これらは `libssruntime` が提供する **Slot API** と **SoA バッファ** を活用することで、複雑な同期ロジックを意識せずに実装できます。

## 1. インスタンスパーツ (Instance)

別のアニメーションを部品として参照し、親の再生時間に同期して再生します。

### Slot API による管理
Player はパーツごとに `ss_instance_slot_create()` で作成した slot を保持しておき、毎フレーム `ss_instance_slot_step()` を呼び出します。

```cpp
// 毎フレームの更新ループ内
const ss_event_instance_info info = ss_runtime_get_active_event_instance(runtime_ctx, p_idx);

// 状態更新と同期
const ss_instance_step_result r = ss_instance_slot_step(
    state.instance_slot, info, child->runtime_ctx,
    child->previous_frame_no, parent_frame_no, delta_seconds, parent_looped);

if (r.transitioned) {
    // インスタンスの切り替わり（イベントの発生）を検知。
    // 子 Player の play() を呼び出し、開始イベントなどを発火させる。
    child->play();
}

if (r.visible) {
    child->render(r.child_frame_no);
}
```

#### 外部 SSAB の解決 (External Instances)
インスタンスパーツが「別ファイルの `.ssab`（別のアニメーションパック）」を参照している場合があります。Player は初期化時に、SSAB の `external_instances()` セクションを確認し、必要な外部ファイルを再帰的にロードして、適切な Context にバインドする責任があります。

## 2. エフェクトパーツ (Effect)

パーティクルエフェクトのシミュレーション結果を、絶対時間（フレーム）に基づいて瞬時に取得できます。

### Effect Slot の活用
`ss_effect_slot_step` を呼び出すと、そのフレームの全パーティクルの描画データ（頂点、UV、色、テクスチャ）が **`EffectDrawPlan`** として返されます。

```cpp
// 毎フレームの更新ループ内
const ss_effect_event_info ev = ss_runtime_get_active_effect_event(runtime_ctx, p_idx);

// シミュレーション実行。
const ss_effect_step_result step = ss_effect_slot_step(
    slot.effect_slot, ev, f.frame_no, f.delta_seconds, fps, f.parent_looped,
    /*y_flip*/ true, /*vert_stride*/ 2);

if (step.visible) {
    auto plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
    // 描画実行...
}
```

## 3. 独立再生 (Independent Playback) の対応

エフェクトやインスタンスには、親アニメーションの再生速度や停止状態に関わらず、独自の経過時間でシミュレーションを継続する「独立再生」設定があります。

- **検知**: `ss_effect_slot_is_independent()` や `ss_instance_slot_is_independent()` が `true` を返す場合、そのパーツは独立再生モードです。
- **更新の継続**: 親アニメーションが一時停止中や、シークによってフレームが動いていない場合でも、これらのパーツは `delta_seconds` を進めて `step` を呼び出し続ける必要があります。これを怠ると、パーティクルが空中で静止するなどの不自然な挙動になります。

## 4. メッシュとデフォーム (Mesh / Deform)

ボーンによるスキンメッシュ変形や、自由な頂点変形をサポートします。

### メッシュ SoA バッファ
メッシュの頂点計算（スキニング含む）は Brain 側で完了しています。Player は **計算済みのワールド頂点座標** を SoA バッファから直接取得して描画するだけです。

```cpp
// メッシュ描画バッチが来たとき
const float *v_x, *v_y, *uvs;
const uint32_t *v_offsets;

ss_runtime_get_mesh_vertices_x(ctx, &v_x, &len);
ss_runtime_get_mesh_vertices_y(ctx, &v_y, &len);
ss_runtime_get_mesh_vertex_offsets(ctx, &v_offsets, &len);

// パーツ p_idx の頂点範囲を特定して描画
uint32_t start = v_offsets[p_idx];
uint32_t end   = v_offsets[p_idx + 1];
DrawMeshPart(v_x + start, v_y + start, uvs + start, ...);
```

## 5. この段階での動作確認 (Validation)

`overall.sspj` の以下のデータを使用して、複雑な構造が正しく再現されるかを確認します。

- **`Instance.ssae`**: 子アニメーションが親の動き・時間に同期してループ再生されているか。外部参照が解決されているか。
- **`Effect.ssae`**: パーティクルが正しく発生し、テクスチャや合成モードが反映されているか。独立再生が機能しているか。
- **`Mesh.ssae`**: 頂点変形やボーン追従によってメッシュが滑らかに歪んでいるか。

---
[目次に戻る](./README.ja.md)
