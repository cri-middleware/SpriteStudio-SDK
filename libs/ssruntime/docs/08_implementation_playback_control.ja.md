# 実装フェーズ 2：再生制御と状態管理 (Playback Control)

アニメーションの再生状態の管理、ループ設定、再生終了の検知など、Player のコントローラーとしての機能を実装します。

## 1. 再生コントロールメソッド
Player の API として、以下のメソッドを用意することを推奨します。これらは内部で libssruntime の対応する関数を呼び出します。

- **Play**: `ss_runtime_play(context)`
- **Pause**: `ss_runtime_pause(context)`
- **Stop**: `ss_runtime_stop(context)`（フレームをリセットして停止）

## 2. 再生設定と取得
アニメーションの挙動をカスタマイズするための設定項目です。

### 基本設定
- **再生速度**: `ss_runtime_set_animation_speed(context, speed)` / `1.0f` が等倍。
- **フレームレート**: `ss_runtime_set_frame_rate(context, fps)` / ssab のデフォルト値以外を使いたい場合に設定。
- **再生範囲**: `ss_runtime_set_animation_section(context, start, end)`

### 再生方向とスタイル
- **再生方向**: `ss_runtime_set_playback_direction(context, direction, style)`
    - `direction`: 0 = 正方向 (Forward), 1 = 逆方向 (Backward)
    - `style`: 0 = 通常 (Normal), 1 = ピンポン (PingPong)
- **ループ回数**: `ss_runtime_set_loop(context, count)`
    - `0` で無限ループ。

## 3. 状態の取得
Player の外部（スクリプト等）から現在の状況を確認するための API です。

- `ss_runtime_is_playing(context)`: 再生中か
- `ss_runtime_get_frame_no(context)`: 現在のフレーム番号（整数）
- `ss_runtime_get_frame_no_decimal(context)`: 現在のフレーム番号（小数：補間計算用）
- `ss_runtime_get_fps(context)`: 設定されている FPS

## 4. 再生終了コールバックの実装
libssruntime は、再生が終了（またはループの終端に到達）した際に内部でフラグを立てます。

- **終了検知**: `ss_runtime_is_end_frame_reached(context)` が `true` になったタイミングを確認します。

### コールバックの発火タイミングの検討
Player 実装者は、用途に合わせて以下のタイミングでイベント（コールバック）を発火させる必要があります。

- **無限ループ時**: 
    - 終端に到達するたびに発火する（1ループ終了ごと）
    - または、発火させない
- **有限ループ時**:
    - 終端に到達するたびに発火する
    - すべてのループが完了した最後のみ発火する

## 5. フレームスキップの設定
`ss_runtime_set_skip_frames(context, bool)` を用いて、処理が追いつかない場合にフレームを飛ばすかどうかを設定できます。
- **ON**: 経過時間（デルタタイム）を優先し、描画が間に合わないフレームをスキップします。
- **OFF**: 毎フレーム確実に進めます。処理が重い場合、アニメーションはスローモーションになります。
