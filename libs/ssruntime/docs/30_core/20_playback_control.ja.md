> **日本語** | [English](./20_playback_control.md)

# 再生制御とイベント処理 (Playback & Events)

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

### `ss_runtime_set_loop` の値

| 値 | 意味 |
| :--- | :--- |
| `n` (> 0) | `n` 周再生して停止する。`1` = 1回のみ |
| `-1` | 無限ループ |
| `0` | 無限ループ（`-1` の別名。0 以下は全て `-1` に正規化される） |

> [!NOTE]
> ループ回数は `setup_animation` が既定値（無限）へ戻します。`setup_animation` の後、`play` の前に再適用してください。`play` はその時点のループ回数でカウンタを再武装するため、完了したアニメを `play` で再生し直すと同じループ回数がもう一度適用されます。

### 再生ヘッドの操作
- **シーク**: `ss_runtime_set_frame_no(context, frame)` で任意のフレームへジャンプできます。
- **一時停止**: `ss_runtime_pause(context)` で再生を一時停止/再開（トグル）します。

## 2. 毎フレームの更新と終了検知

Player は毎フレーム、ゲームエンジンのデルタタイムをライブラリに渡して状態を更新します。

### 実装の型

`looped` / `finished` の通知は、**以下の形で実装してください**。フラグごとに寿命が違うため、扱い方も異なります。

```cpp
class MyPlayer {
    bool prev_finished = false;   // finished の立ち上がりエッジ検出用

    void update(float delta_seconds) {
        // 1. 更新実行 (デルタタイムはミリ秒で渡す)
        ss_runtime_update(context, delta_seconds * 1000.0f);

        // 2. looped は「パルス」。update の直後にそのまま読む
        if (ss_runtime_is_looped(context)) {
            emit_looped();
        }

        // 3. finished は「状態」。立ち上がりエッジで 1 回だけ通知する
        bool finished = ss_runtime_is_finished(context);
        if (finished && !prev_finished) {
            emit_finished();
        }
        prev_finished = finished;
    }

    void play() {
        prev_finished = false;   // 再生し直しに備えてラッチを戻す
        ss_runtime_play(context);
    }
};
```

### 2つのフラグの違い

| API | 意味 | 寿命 | 扱い方 |
| :--- | :--- | :--- | :--- |
| `ss_runtime_is_looped` | 再生ヘッドが**頭に戻った**（＝1周した） | **パルス**。`update` の先頭で毎回クリアされる | そのまま通知 |
| `ss_runtime_is_finished` | 指定ループ回数を**すべて完了した** | **状態**。次の `play` / `setup_animation` まで `true` のまま | **エッジを取って**通知 |

- `is_looped` は 1 回の `update` で複数周した場合も 1 回にまとまります（大きな `delta` ＋ フレームスキップ有効時）。
- `is_finished` は `stop()` では立ちません。指定ループ回数を完走した場合のみです。
- `is_finished` は無限ループ（既定値）では永久に立ちません。無限ループの終了は `looped` を数えるか、Player 側で `stop()` を呼んで判断してください。

### よくある誤り

実際に複数の Player で起きた不具合です。いずれも `is_finished` を上記の型で使えば起きません。

| 誤った実装 | 症状 |
| :--- | :--- |
| エッジを取らずに `is_finished` をそのまま通知する | 完了後、**毎フレーム発火し続ける** |
| `prev_finished` を `play` でリセットしない | 完了 → 再生し直しの際、次の完了を**取りこぼす**ことがある |
| `!is_playing` を `finished` として代用する | `stop()` や再生開始前と区別できず**誤発火**する |
| 「最終フレームに到達したか」で終了を判定する | 有限ループで**毎周発火**する。無限ループでは**一度も発火しない**。再生ループをそこで止めると**1周目で再生が打ち切られる** |

> [!NOTE]
> 「最終フレームを表示中」という内部状態（`end_frame_reached`）は、かつて C-API / WASM に公開されていましたが、**これを終了判定に使った Player がすべてバグを踏んだ**ため、Player 向け API から外しました。終了判定は `is_finished` だけを使ってください。

> [!NOTE]
> `is_playing` が false の間は `update` 自体を早期 return する作りなら、その guard がエッジ検出を兼ねるため `prev_finished` は理屈の上では不要です。しかし依存関係が暗黙になり、後から guard を動かすと壊れます。**上記の型どおり明示的にラッチを持つことを推奨します。**

### イベント列の例

`set_loop(3)` で 0〜4 フレームのアニメを再生した場合:

| 周 | 通知 |
| :--- | :--- |
| 1周目の末尾 → 頭に戻る | `looped` |
| 2周目の末尾 → 頭に戻る | `looped` |
| 3周目の末尾 | `finished`（頭には戻らず最終フレームで停止、`is_playing` が false になる） |

無限ループ（`-1` / `0`）では `looped` のみが繰り返し発火し、`finished` は永久に発火しません。

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
    // Signal の処理 (SpriteStudio v7以降)
    if (events->signals()) {
        for (auto sig : *events->signals()) {
            if (!sig || !sig->value() || !sig->value()->active()) continue;
            
            auto val = sig->value();
            const char* command = val->command_id() ? val->command_id()->c_str() : "";
            
            // パラメータの展開処理 (C++の汎用的な map 等に格納する例)
            std::map<std::string, MyVariant> params_map;
            if (auto params = val->params()) {
                for (uint32_t k = 0; k < params->size(); k++) {
                    auto param = params->Get(k);
                    if (!param || !param->param_id()) continue;
                    std::string p_id = param->param_id()->c_str();

                    // FlatBuffers の Union 型から値を取り出して格納する
                    switch (param->value_type()) {
                        case ss::format::SignalCommandParamValue_Bool:
                            if (auto b = param->value_as_Bool()) params_map[p_id] = b->value();
                            break;
                        case ss::format::SignalCommandParamValue_Index:
                            if (auto i = param->value_as_Index()) params_map[p_id] = i->value();
                            break;
                        case ss::format::SignalCommandParamValue_Integer:
                            if (auto i = param->value_as_Integer()) params_map[p_id] = i->value();
                            break;
                        case ss::format::SignalCommandParamValue_Floating:
                            if (auto f = param->value_as_Floating()) params_map[p_id] = f->value();
                            break;
                        case ss::format::SignalCommandParamValue_Text:
                            if (auto t = param->value_as_Text()) {
                                if (t->value()) params_map[p_id] = t->value()->c_str();
                            }
                            break;
                        default:
                            break;
                    }
                }
            }
            emit_signal("signal", command, params_map);
        }
    }
    
    // Audio の処理
    if (events->audios()) {
        for (auto audio : *events->audios()) {
            if (!audio || !audio->value()) continue;
            
            auto val = audio->value();
            const char* sound_name = val->sound_name() ? val->sound_name()->c_str() : "";
            int loop_num = val->loop_num();
            
            emit_signal("audio", audio->part_index(), sound_name, loop_num);
        }
    }
}
```

## 4. 推奨される Player API (設計指針)

利用者がアニメーションを容易に扱えるよう、以下の API インターフェースを提供することを推奨します。

| カテゴリ | 推奨 API | 内容 |
| :--- | :--- | :--- |
| **基本操作** | `set_animation(name)`, `play(loop)`, `stop()`, `pause()` | アニメーション設定・再生・停止・一時停止 |
| **パラメータ** | `set_speed(f32)`, `set_frame(f32)` | 速度と現在のフレーム位置 |
| **コールバック** | `finished`, `looped`, `user_data`, `signal` | 各種イベント発生時の通知（本章 2. 参照） |

### `set_animation` と `play` の分離（推奨）

アニメーションの**セットアップ**（`set_animation`）と**再生開始**（`play`）は、別メソッドへ分けて公開することを推奨します（Godot / Unity 移植が採用）。内部的には C-API の `ss_runtime_setup_animation` → `ss_runtime_play`（本章 1. 参照）をそのまま 2 つの公開 API に割り当てます。

> **なぜ分けるか（メモリアロケーション）**: `set_animation`（`ss_runtime_setup_animation`）はアニメーションのセットアップ時に ssruntime 内部でヒープアロケーションを伴います。これを `play` の内部で毎回行うと、再生のたびにアロケートが走ってしまいます。**アロケーションを伴うセットアップは再生前に済ませ、`play()` はアロケーションの無い軽い操作に保つ**のが狙いです。

| API | 内容 |
| :--- | :--- |
| `set_animation(name)` | アニメのデータをロード・設定し、先頭フレームで待機する（**再生は開始しない**）。ここで内部アロケーションが発生する |
| `play()` / `play(startFrame)` | セットアップ済みのアニメをその場（または指定フレーム）から再生開始する |

利点:

- `play()` の直前までに重い準備（`setup_animation` や、そのアニメが必要とするアセットのロード）を済ませられるため、再生開始を即時にできる。
- 再生せずに先頭フレームを posed 表示できる（待機ポーズ）。
- 再生中でなくてもアニメを差し替えられる（次に `play()` を呼ぶまで開始しない）。

この設計では、再生開始時の通知として `started` コールバックを併せて提供すると、`set_animation`（差し替え）と `play`（開始）の区別が利用側で扱いやすくなります。

> [!NOTE]
> ループ回数・速度・再生方向などは `setup_animation` が既定値へ戻すため、`setAnimation` 内で（または `play` の前に）再適用してください。

### サブフレーム (Sub-frame) 再生
`libssruntime` はフレーム番号を `f32` で管理しており、整数フレーム間の「補間」が可能です。高リフレッシュレート環境での滑らかな動きを実現するため、Player 側でも `f32` でのフレーム管理をサポートすることを推奨します。

#### 受け付けるフレーム範囲と終端の扱い
`ss_runtime_get_frame_data(frame_no)` が受け付ける `frame_no` は、半開区間 **`[start_frame, end_frame + 1)`** です。これは「フレーム N は連続的な再生タイムライン上で時間区間 `[N, N+1)` を占有する」という考え方に対応します。

- サブフレーム再生では、終端・ループ境界で再生ヘッドが一瞬 `(end_frame, end_frame + 1)` の小数値になります。この区間は **有効** で、最終キーフレームの値を保持したポーズ（`end_frame` ちょうどと同一）が返ります。終端を越えた外挿は行われません。
- 範囲外（`frame_no < start_frame` / `frame_no >= end_frame + 1` / `NaN`）は `InvalidParameter` を返します。この場合 **FrameData が得られず、そのフレームの描画が消える**（全パーツが一瞬消失する）ため注意してください。

> [!NOTE]
> サブフレーム再生時に、終端での消失を避ける目的で `frame_no` を `end_frame` へクランプする必要は **ありません**。ランタイムが末尾区間 `(end_frame, end_frame + 1)` を正しく扱います。
>
> ただしこの範囲は **アニメーションデータ全体（クリップ）** の範囲です。`set_animation_section` で再生区間を限定している場合、ランタイムはセクション端へのクランプを行わず、セクションを越えてもクリップ内の実キーフレームを評価します。セクション再生とサブフレームを併用する場合は、表示用に **Player 側で `frame_no` をセクション端へクランプ** してください。

### 複数アニメーションのブレンド (State machine / Locomotion blend)
クロスフェードやアニメーションの重ね合わせを実装する場合は [アニメーションミキシング](../40_components/60_animation_mixing.ja.md) を参照。トラックと `alpha`、加算オーバーレイ、1トラックに複数アニメを weight で混ぜる方法を解説します。

---
[目次に戻る](../README.ja.md)
