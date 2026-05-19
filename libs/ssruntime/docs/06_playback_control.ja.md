# 実装フェーズ 2：再生制御とイベント処理 (Playback & Events)

アニメーションの再生状態（再生・停止・ループ）の管理と、タイムライン上のイベント（ユーザーデータ・シグナル）をゲームロジックへ通知する仕組みを実装します。

## 1. アニメーションのセットアップと再生

`libssruntime` コンテキストに対して、再生したいアニメーションを名前で指定し、再生を開始します。

```cpp
// 1. アニメーションのセットアップ
ss_runtime_setup_animation(context, "attack_pose");

// 2. 再生設定 (ループ回数: -1=無限, 1=1回のみ)
ss_runtime_set_loop(context, -1);

// 3. 再生開始
ss_runtime_play(context);
```

### 再生ヘッドの操作
- **シーク**: `ss_runtime_set_frame_no(context, frame)` で任意のフレームへジャンプできます。
- **一時停止**: `ss_runtime_pause(context)` で再生を一時停止/再開（トグル）します。

## 2. 毎フレームの更新と終了検知

Player は毎フレーム、ゲームエンジンのデルタタイムをライブラリに渡して状態を更新します。

```cpp
// 更新実行 (デルタタイムはミリ秒で渡す)
ss_runtime_update(context, delta_ms);

// 状態確認
if (ss_runtime_is_end_frame_reached(context)) {
    // 再生終了（非ループ時、または有限ループの完了）
    on_animation_finished();
}
if (ss_runtime_is_looped(context)) {
    // ループ境界を跨いだ
    on_animation_looped();
}
```

## 3. イベントの取得 (ポーリング)

`libssruntime` は更新中に通過したイベント（ユーザーデータ、シグナル、サウンド）を収集します。Player は `update` の直後にこれらを一括取得（ポーリング）します。

```cpp
// 通過したイベント数を取得
int count = ss_runtime_get_passed_event_count(context);

for (int i = 0; i < count; i++) {
    // イベントデータのインデックスを取得
    int event_idx = ss_runtime_get_passed_event_index(context, i);
    
    // SSAB (FlatBuffers) から具体的なデータを参照
    auto events = ssab_binary->events()->Get(event_idx);
    
    // UserData の処理
    if (events->users()) {
        for (auto user : *events->users()) {
            emit_signal("user_data", user->part_index(), user->value());
        }
    }
    // Signal / Audio も同様に処理
}
```

## 4. 推奨される Player API (設計指針)

利用者がアニメーションを容易に扱えるよう、以下の API インターフェースを提供することを推奨します。

| カテゴリ | 推奨 API | 内容 |
| :--- | :--- | :--- |
| **基本操作** | `play(name, loop)`, `stop()`, `pause()` | 再生・停止・一時停止 |
| **パラメータ** | `set_speed(f32)`, `set_frame(f32)` | 速度と現在のフレーム位置 |
| **コールバック** | `finished`, `looped`, `user_data`, `signal` | 各種イベント発生時の通知 |

### サブフレーム (Sub-frame) 再生
`libssruntime` はフレーム番号を `f32` で管理しており、整数フレーム間の「補間」が可能です。高リフレッシュレート環境での滑らかな動きを実現するため、Player 側でも `f32` でのフレーム管理をサポートすることを推奨します。

---
[目次に戻る](./README.ja.md)
