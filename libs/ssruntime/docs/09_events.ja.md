# 実装フェーズ 6：イベント処理 (UserData / Signal / Audio)

SpriteStudio 7 では、アニメーションのタイムライン上の特定のフレームに各種イベントを配置できます。
`libssruntime` は、フレームスキップやループ再生が発生した場合でも、通過したすべてのイベントを漏れなく収集し、Player 側に通知する仕組みを提供します。

> **【重要】Instance と Effect の再生について**
> 以前は Instance（子アニメーション）や Effect（パーティクル）の再生開始をイベント駆動で制御する手法も存在しましたが、**逆再生やシーク（特定フレームへのジャンプ）に正しく対応するため、現在のアーキテクチャではこれらを「毎フレームの描画ループ内で時間を同期計算して描画する」アプローチを強く推奨しています。** 詳細は [実装フェーズ 5：拡張機能の実装](./08_advanced_rendering.ja.md) を参照してください。
> 本章で扱う「イベント」は、主に一過性のトリガーとして発火するユーザーデータやサウンドの再生要求を対象としています。

## 1. イベント取得の仕組み

`libssruntime` は更新（`ss_runtime_update`）の過程で論理的に通過したすべてのフレームを内部で記録します。
Player 側は `update` 呼び出し後に、その更新で発生したイベントを**ポーリング（一括取得）**形式で受け取ります。

これにより、Player 側で「前回と今回のフレーム差分」を計算したり、ループ境界の判定（終端から先頭に戻る際の処理など）を行ったりする必要がなくなり、常に正しい順序でイベントを処理できます。

## 2. Player 側での実装手順

Player 側は、`ss_runtime_update` を呼び出した直後にイベント総数を取得し、ループを回してそれぞれのイベントデータにアクセスします。
データの詳細は、Player 側が保持している SSAB データ（FlatBuffers のテーブル）から直接ルックアップします。

```cpp
// 1. アニメーションの更新
int frame_no = ss_runtime_update(runtime_ctx, delta_time);

if (_currentAnimationData->events() != nullptr) {
    // 2. 今回の更新で発生したイベント（イベントを含むフレーム）数を取得
    int event_count = ss_runtime_get_passed_event_count(runtime_ctx);

    // 3. イベントごとのポーリング処理
    for (int i = 0; i < event_count; i++) {
        // データのインデックスを直接取得
        int event_idx = ss_runtime_get_passed_event_index(runtime_ctx, i);

        // 自分の保持している AnimationData からインデックスを指定して直接アクセス
        auto events_per_frame = _currentAnimationData->events()->Get(event_idx);

        // ユーザーデータ (UserData)
        if (auto users = events_per_frame->users()) {
            for (auto user : *users) {
                // 文字列や数値などのデータを取得し、ゲーム側のイベントを発火
                emit_user_data_event(user->part_index(), user->value());
            }
        }

        // シグナル (Signal)
        if (auto signals = events_per_frame->signals()) {
            for (auto signal : *signals) {
                // シグナル名（コマンド）を取得し、ゲーム側のイベントを発火
                emit_signal_event(signal->part_index(), signal->value());
            }
        }

        // 音声 (Audio)
        if (auto audios = events_per_frame->audios()) {
            for (auto audio : *audios) {
                // 音声ファイル名等を取得し、エンジンのオーディオシステムで再生
                emit_audio_event(audio->part_index(), audio->value());
            }
        }
    }
}
```

## 3. 各イベントの役割と用途

### ユーザーデータ (UserData)
アニメーションに埋め込まれた数値、座標、矩形、文字列などの汎用データです。
- **用途**: アニメーションの特定タイミングでの当たり判定の有効化/無効化、カメラ揺れ（スクリーンシェイク）の発生タイミング、スクリプト関数の呼び出し指定など、ゲームロジックとの連携に最も多用されます。

### シグナル (Signal)
特定のコマンド名とパラメータを外部に通知するための仕組みです。
- **用途**: 状態遷移の明確なトリガーとして使用します（例：「AttackEnd」シグナルで攻撃アニメーションから待機状態へ移行するなど）。UserDataよりも文字列ベースのコマンド通知に特化しています。

### サウンド (Audio)
SE（効果音）の再生指示です。
- **用途**: 指定したフレームでの SE 再生。`libssruntime` は音声の再生機能を内包していないため、Player 側でゲームエンジンのオーディオ管理システムと連携させて実際の再生を実行します。

## 4. 推奨されるコールバック API

Player は、利用者がこれらのイベントを容易に検知できるように、以下のようなコールバック API を提供することを推奨します。

- **setAnimationFinishedCallback(callback)**: アニメーションの再生が終了（またはループの終端に到達）した際の通知。
    - **判定方法**: イベントとしてではなく、`ss_runtime_is_end_frame_reached(context)` が `true` になったタイミングで発火させます。
- **setUserDataCallback(callback)**: タイムライン上のユーザーデータ（UserData）を検出した際の通知。
- **setSignalCallback(callback)**: シグナルを受信した際の通知。
- **setAudioCallback(callback)**: 音声再生要求が発生した際の通知。

## 注意点

- **発火順序**: イベントは時間軸に沿った正しい順序（順再生なら昇順、逆再生なら降順、ループなら終端→開始の順）で格納されています。
- **同一フレーム内の複数イベント**: 同じフレームに複数のパーツがイベントを持っている場合、それらもすべてリストに含まれます。`part_index()` を参照して、どのパーツからのイベントかを確認してください。

## 5. この段階での動作確認 (Validation)

時間経過に伴うイベントの取得が正しいかを確認します。

- **確認対象**: `UserData.ssae`
    - **チェック内容**: 指定されたフレームで期待通りのユーザーデータ（文字列、数値等）が正しく取得でき、コールバックが呼ばれるか。
- **確認対象**: `Signal.ssae`
    - **チェック内容**: シグナルコマンドの受信とパラメータのパースが正しく行えているか。
- **確認対象**: `Sound.ssae` (※オプション)
    - **チェック内容**: フレームの進行に合わせて音声再生要求が発火しているか。

---
[目次に戻る](./README.ja.md)
