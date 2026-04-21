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

// 2. 今回の更新でイベントが発生したフレーム数を取得
int event_frame_count = ss_runtime_get_passed_event_count(runtime_ctx);

// 3. イベントが発生したフレームごとのポーリング処理
for (int i = 0; i < event_frame_count; i++) {
    int frame = ss_runtime_get_passed_event_frame_no(runtime_ctx, i);

    // 自分の保持している AnimationData から対象フレームのイベントを検索
    auto events_per_frame = _currentAnimationData->events()->LookupByKey(frame);
    if (!events_per_frame) continue;

    // ユーザーデータ (UserData)
    if (auto users = events_per_frame->users()) {
        for (auto user : *users) {
            emit_user_data_event(user->part_index(), user->value());
        }
    }

    // シグナル (Signal)
    if (auto signals = events_per_frame->signals()) {
        for (auto signal : *signals) {
            emit_signal_event(signal->part_index(), signal->value());
        }
    }

    // 音声 (Audio)
    if (auto audios = events_per_frame->audios()) {
        for (auto audio : *audios) {
            emit_audio_event(audio->part_index(), audio->value());
        }
    }

    // エフェクト等、その他のイベントも同様に処理
    // ...
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
- **用途**: インスタンスパーツの再生・停止・同期を Player 側でハンドリングするために使用します。`libssruntime` は自動的な同期を行わなくなったため、Player はこのイベントをフックして、子アニメーション用の Context を作成・更新する責務を持ちます。同期計算には `ss_util_calculate_instance_frame` を利用できます。

### エフェクト (Effect)
パーティクルエフェクトパーツに対する制御イベントです。
- **用途**: エフェクトの再生開始、停止、あるいは特定パラメータの変更タイミングを検知します。Player はこのイベントを受け取ったタイミングで `ss_effect_create` を呼び出し、シミュレーションを開始する責務を持ちます。

## 特殊なイベント処理の補助 (Instance / Effect)

Instance と Effect に関しては、Player 側の実装を容易にするためのユーティリティ関数が提供されています。

### Instance の同期計算
子アニメーションの再生フレームを算出するために、以下の関数を利用することを推奨します。
- **ss_util_calculate_instance_frame(...)**: 親のフレーム、独立動作フラグ、オフセット等を渡すと、SpriteStudio の仕様に基づいた正しい再生フレームを算出します。

### Effect の管理
エフェクトのライフサイクル管理のために、以下の関数が利用可能です。
- **ss_effect_create(...)**: エフェクトシミュレータ（EffectContext）の生成。
- **ss_effect_update(...)**: シミュレーションの時間を進める。
- **ss_effect_get_state(...)**: 最新のパーティクル状態（FlatBuffers）の取得。
- **ss_effect_reset(...)**: Context を再利用するための初期化。
- **ss_effect_get_lifetime(...)**: エフェクトが消滅するまでの総フレーム数の取得。
- **ss_effect_destroy(...)**: Context の破棄。

Player はこれらの API をイベント駆動で呼び出すことで、複雑なエフェクト・インスタンスの挙動を完全に制御できます。


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

## この段階での動作確認 (Validation)

時間経過に伴うイベントの取得が正しいかを確認します。

- **確認対象**: `UserData.ssae`
    - **チェック内容**: 指定されたフレームで期待通りのユーザーデータ（文字列、数値等）が正しく取得できるか。
- **確認対象**: `Signal.ssae`
    - **チェック内容**: シグナルコマンドの受信とパラメータのパースが正しく行えているか。

---
[目次に戻る](./README.ja.md)
