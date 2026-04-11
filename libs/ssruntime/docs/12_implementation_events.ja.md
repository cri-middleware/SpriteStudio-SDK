# 実装フェーズ 6：イベントシステムの実装

アニメーション中に発生する「ユーザーデータ」「シグナル」「サウンド」などのイベントを Player 側で処理する方法です。

## 1. イベントデータの取得方法
libssruntime の `update` で時間を進める際、フレームスキップが発生したり、再生方向が変わったりすることがあります。

- **保存すべき状態**: `previous_frame_no` を保持しておき、今回の `current_frame_no` と比較します。
- **走査の範囲**: `previous_frame_no` から `current_frame_no` までの間にあるすべてのフレームをループでチェックします。
- **ループ時の注意**: 逆再生やループを跨ぐ場合（`current < previous`）の走査順序にも注意が必要です。

## 2. イベントデータの検索 (FlatBuffers)
libssruntime の計算済み `PartState` にはイベント情報は含まれません。Player 側で保持している `AnimationData`（初期化時に取得したもの）から直接イベントを検索します。

```cpp
// Godot (C++) でのイベント走査の例
void updateEvents(int from_frame, int to_frame) {
    auto events = current_animation_data->events();
    if (!events) return;

    for (int i = from_frame; i <= to_frame; i++) {
        // FlatBuffers の LookupByKey を使用して、そのフレームのイベントを高速検索
        auto eventsPerFrame = events->LookupByKey(i);
        if (!eventsPerFrame) continue;

        // ユーザーデータの処理
        if (auto users = eventsPerFrame->users()) {
            for (auto user : *users) {
                // Godot のシグナルを発火させる例
                emit_signal("user_data", user->flag(), user->int_value(), ...);
            }
        }

        // シグナルの処理
        if (auto signals = eventsPerFrame->signals()) {
            for (auto sig : *signals) {
                emit_signal("signal", String(sig->command()->c_str()), ...);
            }
        }
    }
}
```

## 3. 各イベントの役割

### ユーザーデータ (UserData)
アニメーションに埋め込まれた数値、座標、矩形、文字列などの汎用データです。
- **用途**: 当たり判定の有効化、エフェクトの発生タイミング、特定のスクリプト実行など。

### シグナル (Signal)
特定のコマンド名とパラメータを外部に通知するための仕組みです。
- **用途**: 状態遷移のトリガー（「攻撃終了」シグナルで待機状態へ移行するなど）、システムへの通知。

### サウンド (Audio)
SE（効果音）の再生指示です。
- **用途**: 指定したフレームでの SE 再生。Player 側でエンジンのオーディオ管理システムと連携させて再生を実行します。

## 4. イベント通知 API (コールバック)
Player は、利用者がこれらのイベントを検知できるように、以下のコールバック API を提供することを推奨します。

- **setAnimationFinishedCallback(callback)**: アニメーションの再生が終了（またはループの終端に到達）した際の通知を受け取ります。
- **setUserDataCallback(callback)**: タイムライン上のユーザーデータ（UserData）を検出した際の通知を受け取ります。

## 5. この段階での動作確認 (Validation)
時間経過に伴うイベントの取得が正しいかを確認します。

- **確認対象**: `UserData.ssae`
    - **チェック内容**: 指定されたフレームで期待通りのユーザーデータ（文字列、数値等）がシグナルやコールバックとして発火するか。
- **確認対象**: `Signal.ssae`
    - **チェック内容**: シグナルコマンドの受信とパラメータのパースが正しく行えているか。
