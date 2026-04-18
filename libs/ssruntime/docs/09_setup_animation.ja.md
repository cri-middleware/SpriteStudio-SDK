# 実装フェーズ 2：アニメーション指定 (Setup Animation)

再生するアニメーションの選択や、再生オプションの設定を行うフェーズです。

## 1. アニメーションのセットアップ
ssab 内に含まれる複数のアニメーションの中から、再生したいアニメーションを名前で指定します。

```cpp
// 指定したアニメーション名が存在するか確認し、準備する
const char* anime_name = "anime_1";
bool success = ss_runtime_setup_animation(context, anime_name);
```

## 2. 再生設定
アニメーションのループ回数や速度、再生方向などを設定します。

```cpp
// ループ回数（0 = 無限ループ, 1 = 1回再生, n = n回再生）
ss_runtime_set_loop(context, 0);

// 再生速度（1.0 = 等倍, 2.0 = 2倍速, 0.5 = 半速）
ss_runtime_set_animation_speed(context, 1.0f);

// 再生方向とスタイル（0 = 正方向, 1 = 逆方向 / 0 = 通常, 1 = ピンポン）
ss_runtime_set_playback_direction(context, 0, 0);

// 指定したフレーム範囲のみ再生する場合
ss_runtime_set_animation_section(context, 0, 30);
```

## 3. 再生開始・制御
アニメーションの再生、停止、一時停止を行います。

```cpp
// 再生開始
ss_runtime_play(context);

// 特定のフレームから再生
// ss_runtime_play_with_start_frame(context, 10);

// 一時停止 / 停止（フレームリセット）
// ss_runtime_pause(context);
// ss_runtime_stop(context);
```

## 4. 状態の確認
現在の再生状況を取得します。

```cpp
// 再生中か？
bool is_playing = ss_runtime_is_playing(context);

// 最終フレームに到達したか？
bool is_end = ss_runtime_is_end_frame_reached(context);
```

---
[目次に戻る](./README.ja.md)
