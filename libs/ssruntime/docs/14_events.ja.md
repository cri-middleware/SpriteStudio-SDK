# 14. イベント処理 (UserData / Signal / Audio / Instance / Effect)

SpriteStudio 7 では、アニメーションの特定のフレームに各種イベントを配置できます。
`libssruntime` は、フレームスキップやループ再生が発生した場合でも、通過したすべてのイベントを漏れなく収集し、Player 側に通知する仕組みを提供します。

## 仕組み

`libssruntime` は更新（`ss_runtime_update`）の過程で論理的に通過したすべてのフレームを内部で記録します。
Player 側は `update` 呼び出し後に、その更新で発生したイベントを**ポーリング（一括取得）**形式で受け取ります。

これにより、Player 側で「前回と今回のフレーム差分」を計算したり、ループ境界の判定（終端から先頭に戻る際の処理など）を行ったりする必要がなくなり、常に正しい順序でイベントを処理できます。

## Player 側での実装手順

Player 側は、`ss_runtime_update` を呼び出した直後にイベント総数を取得し、ループを回してそれぞれのイベントデータにアクセスします。
データの詳細は、Player 側が保持している SSAB データ（FlatBuffers のテーブル）から直接ルックアップします。

```cpp
// 1. アニメーションの更新
int frame_no = ss_runtime_update(runtime_ctx, delta_time);

// 2. 今回の更新で発生したイベント数を取得
int event_count = ss_runtime_get_passed_event_count(runtime_ctx);

// 3. イベントのポーリング処理
for (int i = 0; i < event_count; i++) {
    int type = ss_runtime_get_passed_event_type(runtime_ctx, i);
    int frame = ss_runtime_get_passed_event_frame_no(runtime_ctx, i);
    int event_idx = ss_runtime_get_passed_event_index(runtime_ctx, i); // 配列内のインデックス
    int part_idx = ss_runtime_get_passed_event_part_index(runtime_ctx, i);

    // 自分の保持している AnimationData から対象フレームのイベントを検索
    auto events_per_frame = _currentAnimationData->events()->LookupByKey(frame);
    if (!events_per_frame) continue;

    switch (type) {
        case 0: // UserData
        {
            auto data = events_per_frame->users()->Get(event_idx);
            // 推奨：UserData 用のコールバックを発火
            emit_user_data_event(part_idx, data);
            break;
        }
        case 1: // Signal
        {
            auto signal = events_per_frame->signals()->Get(event_idx);
            // 推奨：シグナルコマンドの処理
            handle_signal_event(part_idx, signal);
            break;
        }
        case 2: // Audio
        {
            auto audio = events_per_frame->audios()->Get(event_idx);
            // 推奨：音声再生トリガー
            play_audio(audio);
            break;
        }
        case 3: // Instance
        case 4: // Effect
        {
            // 必要に応じて開始タイミングなどを処理
            break;
        }
    }
}
```

## 各イベントの役割

### ユーザーデータ (UserData)
アニメーションに埋め込まれた数値、座標、矩形、文字列などの汎用データです。
- **用途**: 当たり判定の有効化、エフェクトの発生タイミング、特定のスクリプト実行など。

### シグナル (Signal)
特定のコマンド名とパラメータを外部に通知するための仕組みです。
- **用途**: 状態遷移のトリガー（「攻撃終了」シグナルで待機状態へ移行するなど）、システムへの通知。

### サウンド (Audio)
SE（効果音）の再生指示です。
- **用途**: 指定したフレームでの SE 再生。Player 側でエンジンのオーディオ管理システムと連携させて再生を実行します。

### インスタンス (Instance)
インスタンスパーツ（他のアニメーションを入れ子で再生するパーツ）に対する制御イベントです。
- **用途**: 特定のフレームで子アニメーションの再生位置を強制的に変更したり、独立動作を開始させたりする場合に使用します。通常は `libssruntime` が自動的に同期を行いますが、イベントとしてフックすることで Player 独自の制御を差し込むことが可能です。

### エフェクト (Effect)
パーティクルエフェクトパーツに対する制御イベントです。
- **用途**: エフェクトの再生開始、停止、あるいは特定パラメータの変更タイミングを検知します。

## 推奨されるコールバック API

Player は、利用者がこれらのイベントを容易に検知できるように、以下のようなコールバック API を提供することを推奨します。

- **setAnimationFinishedCallback(callback)**: アニメーションの再生が終了（またはループの終端に到達）した際の通知。
    - **判定方法**: `ss_runtime_is_end_frame_reached(context)` が `true` になったタイミングで発火させます。
- **setUserDataCallback(callback)**: タイムライン上のユーザーデータ（UserData）を検出した際の通知。
- **setSignalCallback(callback)**: シグナルを受信した際の通知。
- **setAudioCallback(callback)**: 音声再生要求が発生した際の通知。

## 注意点

- **発火順序**: イベントは時間軸に沿った正しい順序（順再生なら昇順、逆再生なら降順、ループなら終端→開始の順）で格納されています。
- **同一フレーム内の複数イベント**: 同じフレームに複数のパーツがイベントを持っている場合、それらもすべてリストに含まれます。`part_idx` を参照して、どのパーツからのイベントかを確認してください。
- **データの有効性**: `ss_runtime_get_passed_event_index` で返されるインデックスは、必ずそのフレームの該当配列の範囲内にあることが保証されます。

## この段階での動作確認 (Validation)

時間経過に伴うイベントの取得が正しいかを確認します。

- **確認対象**: `UserData.ssae`
    - **チェック内容**: 指定されたフレームで期待通りのユーザーデータ（文字列、数値等）が正しく取得できるか。
- **確認対象**: `Signal.ssae`
    - **チェック内容**: シグナルコマンドの受信とパラメータのパースが正しく行えているか。

---
[目次に戻る](./README.ja.md)
