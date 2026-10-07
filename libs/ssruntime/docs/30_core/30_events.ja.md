> **日本語** | [English](./30_events.md)

# イベント

タイムライン上のキー（ユーザーデータ・シグナル・オーディオ）を、ゲームのコードへ通知する手順です。Player が公開するイベントの名前と集合は [Player API 規約 §4](../20_design/30_api_conventions.ja.md#4-イベント) で決まっています。

## 1. 仕組み

`ss_runtime_update` は、その呼び出しで通過したフレームを記録します。Player は `update` の直後に、通過したフレームのイベントを一覧で受け取ります。

- Brain が返すのは、ssab の中のイベントの**位置**です。イベントの中身（ユーザーデータの値、シグナルのパラメータ、サウンド名）は、Player が ssab から読みます。
- 位置は、再生しているアニメーションの `AnimationData.events()` の添字です。ssab の最上位にイベントの表はありません。
- 1 回の `update` で複数のフレームを通過した場合は、そのすべてのフレームのイベントが返ります。

## 2. イベントを読む

```cpp
ss_runtime_update(ctx, delta_ms);

// track 0 が再生しているアニメーションのイベント表。ミキシングを使う場合はイベントごとに引く（§4）
const auto* events = current_animation->events();
if (events != nullptr) {
    int count = ss_runtime_get_passed_event_count(ctx);
    for (int i = 0; i < count; i++) {
        int event_idx = ss_runtime_get_passed_event_index(ctx, i);
        if (event_idx < 0 || (uint32_t)event_idx >= events->size()) continue;
        auto per_frame = events->Get(event_idx);

        // キーが打たれたフレーム。通過に気づいたフレームと同じとは限らない
        int frame_no = per_frame->frame_index();

        // ユーザーデータ
        if (auto users = per_frame->users()) {
            for (auto user : *users) {
                emit_user_data(user->part_index(), user->value());
            }
        }

        // シグナル
        if (auto signals = per_frame->signals()) {
            for (auto sig : *signals) {
                auto val = sig->value();
                if (!val->active()) continue;
                const char* command = val->command_id() ? val->command_id()->c_str() : "";

                std::map<std::string, MyVariant> params;
                if (auto list = val->params()) {
                    for (auto param : *list) {
                        if (!param->param_id()) continue;
                        std::string id = param->param_id()->str();
                        switch (param->value_type()) {
                            case ss::format::SignalCommandParamValue_Bool:
                                params[id] = param->value_as_Bool()->value(); break;
                            case ss::format::SignalCommandParamValue_Index:
                                params[id] = param->value_as_Index()->value(); break;
                            case ss::format::SignalCommandParamValue_Integer:
                                params[id] = param->value_as_Integer()->value(); break;
                            case ss::format::SignalCommandParamValue_Floating:
                                params[id] = param->value_as_Floating()->value(); break;
                            case ss::format::SignalCommandParamValue_Text:
                                if (auto t = param->value_as_Text()->value()) params[id] = t->str();
                                break;
                            default: break;
                        }
                    }
                }
                emit_signal(sig->part_index(), command, params);
            }
        }

        // オーディオ
        if (auto audios = per_frame->audios()) {
            for (auto audio : *audios) {
                auto val = audio->value();
                const char* sound_name = val->sound_name() ? val->sound_name()->c_str() : "";
                emit_audio(audio->part_index(), sound_name, val->loop_num());
            }
        }
    }
}
```

`emit_*` は Player がホストに通知する処理を表す、実在しない関数です。オーディオのイベントを受けて音を鳴らす手順は [オーディオ](../50_features/50_audio.ja.md) にあります。

## 3. イベントが報告されるフレーム

| 操作 | 次の `update` で報告されるフレーム |
| :--- | :--- |
| 通常の再生 | 前回の `update` の後から今回までに通過したフレーム |
| `ss_runtime_play` / `ss_runtime_play_with_start_frame` | 開始フレーム（とそれ以降に通過したフレーム） |
| `ss_runtime_set_frame_no`（シーク） | 移動先のフレーム。**移動元との間のフレームは報告しない** |
| `ss_runtime_set_animation_section` | 区間の先頭（逆方向なら末尾）のフレーム |

逆方向の再生でも、通過したフレームのイベントは報告されます。

### 3-1. 開始フレームのイベント

`play` の後の最初の `update` は、開始フレーム（たとえばフレーム 0）を通過したフレームとして報告します。フレーム 0 に置いたキーは、この仕組みで最初の再生から発火します。

**イベントの処理を、描画のための条件で止めないでください。** 「整数のフレーム番号が前回から変わっていなければ描画を省く」という最適化の中でイベントも処理すると、最初の `update` で整数のフレームが進まなかったとき（最初の経過時間が短いときなど）、開始フレームのイベントを取りこぼします。取りこぼすかどうかは最初の経過時間しだいなので、「実行すると鳴るがエディタのプレビューでは鳴らない」のような、再現しにくい不具合になります。Brain が報告したイベントは、描画するかどうかにかかわらず処理してください。

### 3-2. シークで飛ばしたイベント

シークは、移動元と移動先の間のイベントを報告しません。間のイベントも通知したい場合は、Player でシークの代わりに `update` を細かく分けて呼ぶか、間のフレームのイベントを ssab から読んで通知してください。

再生中にシークすると、次の `update` で移動先のフレームのイベント（オーディオを含む）が報告されます。シークで音を鳴らしたくない場合は [オーディオ §3](../50_features/50_audio.ja.md#3-シークで鳴らさない) を参照してください。

## 4. 複数のアニメーションを重ねているとき

アニメーションミキシングでは、既定で track 0 のアニメーションのイベントだけが報告されます。他のソースのイベントも報告させるには、そのソースの閾値を下げます（[アニメーションミキシング §6](../50_features/10_animation_mixing.ja.md#6-イベント)）。

`ss_runtime_get_passed_event_index` の値は、**発火したソースのアニメーション**のイベント表の添字です。`ss_runtime_get_passed_event_source_handle(ctx, i)` で発火したソースのハンドルを取り、`ss_runtime_get_source_animation_index(ctx, handle)` でそのソースが再生しているアニメーション（ssab の `animations()` の添字）を調べ、そのアニメーションの `events()` を引いてください。

**track 0 も同じです。** track 0 をクロスフェードすると、トラックは入ってくるアニメーションに引き継がれます。そのため完了後は、`ss_runtime_setup_animation` のときに控えたイベント表では引けません。別のイベントとして報告されるか、範囲外として捨てられます。表を 1 つ控えておくだけで済むのは、クロスフェードもミキシングも使わない Player だけです。

---
[目次に戻る](../README.ja.md)
