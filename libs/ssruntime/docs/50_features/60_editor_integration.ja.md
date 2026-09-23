> **日本語** | [English](./60_editor_integration.md)

# エディタ統合

エンジンのエディタで sspj を読み込み、ssab に変換する仕組みの作り方です。`libssconverter` をエディタの拡張に組み込み、**ディレクトリ単位でまとめて変換する**機能と、**進捗を表示してキャンセルできる**機能を作ります。SSPlayerForGodot と SSPlayerForUnity の実装に基づいています。

## 1. 全体の構成

`libssconverter` の C-API は、別のスレッドで処理する非同期の設計です。エディタの UI（メインスレッド）を止めないよう、次の形で使います。

1. **開始する**: `ss_converter_create` で Context を作り、処理を始めます（`ss_converter_convert` または `ss_converter_scan_dir`）。関数はすぐに戻ります。
2. **毎フレーム確かめる**: エディタの毎フレームの処理で `ss_converter_is_finished` を確かめ、進捗の表示を更新します。
3. **破棄する**: 終わったら結果を受け取り、`ss_converter_destroy` で解放します。

## 2. ディレクトリ単位でまとめて変換する

利用者がフォルダをエディタにドロップしたときなどに、その中の sspj をまとめて探して変換します。

1. `ss_converter_scan_dir` で、別のスレッドでの探索を始めます。
2. 毎フレーム、`ss_converter_scan_found` と `ss_converter_scan_visited` で「探索中… n 件」のような進捗を表示します。
3. 探索が終わったら、`ss_converter_scan_result_count` と `ss_converter_scan_result_at` で、見つかった sspj のパスを集めます。

たくさんの sspj を一度にすべて `ss_converter_convert` すると、CPU の負荷とメモリの使用量が跳ね上がります。同時に動かす Context の数を CPU のコア数程度（4〜8 など）に抑え、空いたところに次の変換を足していく形にしてください。

## 3. キャンセル

大きなプロジェクトの探索や変換では、途中でキャンセルできることが欠かせません。

キャンセルされたら、動いているすべての Context に **`ss_converter_abort(ctx)`** を呼びます。別のスレッドが中断の要求を受け取ります。

> [!WARNING]
> **`ss_converter_abort` の直後に `ss_converter_destroy` を呼ばないでください。** スレッドが止まるまでには少し時間がかかります。毎フレームの確認を続け、**`ss_converter_is_finished(ctx)` が真になってから `ss_converter_destroy(ctx)` を呼んでください。** 守らないと、解放済みのメモリを読んでクラッシュします。

### 3-1. Unity（C#）の例

`EditorApplication.update` で毎フレーム確かめ、`Progress` API のキャンセルを受け取ります。

```csharp
// キャンセル（Progress の UI から呼ばれる）
private static bool OnCancel() {
    _canceled = true;
    _pendingJobs.Clear(); // 待っている変換を捨てる
    unsafe {
        foreach (var ctxPtr in _activeContexts) {
            var ctx = (SpriteStudio.SsConverter.Context*)ctxPtr;
            // 動いている Context に中断を要求する
            SpriteStudio.SsConverter.NativeMethods.ss_converter_abort(ctx);
        }
    }
    return true; // ここでは破棄しない。毎フレームの確認で回収する
}

// 毎フレームの確認
private static void Poll() {
    for (int i = _activeContexts.Count - 1; i >= 0; i--) {
        unsafe {
            var ctx = (SpriteStudio.SsConverter.Context*)_activeContexts[i];

            // 終わるまで待つ（中断を要求した後も、ここで終わるのを待つ）
            if (!SpriteStudio.SsConverter.NativeMethods.ss_converter_is_finished(ctx)) continue;

            try {
                // 結果を処理する
            } finally {
                // 例外が起きても、終わった Context は必ず破棄する
                SpriteStudio.SsConverter.NativeMethods.ss_converter_destroy(ctx);
                _activeContexts.RemoveAt(i);
            }
        }
    }
}
```

### 3-2. Godot（C++）の例

`NOTIFICATION_PROCESS`（毎フレームの処理）で、進捗のダイアログと連携します。

```cpp
void SSImporter::_notification(int p_what) {
    if (p_what != NOTIFICATION_PROCESS) return;

    if (_is_converting) {
        // 進捗のダイアログでキャンセルが押されたか
        if (!_convert_canceling && _import_dialog->is_canceled()) {
            _convert_canceling = true;
            for (int i = 0; i < _active_ctx.size(); i++) {
                ss_converter_abort((Context *)_active_ctx[i]); // 中断を要求する
            }
        }

        // 動いている変換を確かめる
        for (int i = _active_ctx.size() - 1; i >= 0; i--) {
            Context *ctx = (Context *)_active_ctx[i];

            // 中断を要求した後も、終わるまで待つ
            if (!ss_converter_is_finished(ctx)) continue;

            CConverterError result = ss_converter_get_result(ctx);
            if (result == CConverterError::Success) {
                // ... 成功したときの処理 ...
            }

            // スレッドが終わったので、安全に破棄できる
            ss_converter_destroy(ctx);
            _active_ctx.remove_at(i);
        }
    }
}
```

## 4. 例外が起きても回収する

例外やエラーで毎フレームの確認が止まると、別のスレッドと Context のメモリが解放されないまま残ります。結果の処理（C# でのファイルの列挙など）で例外が起きても、`try` / `finally` で必ず `ss_converter_destroy` を呼び、確認の処理を止めないでください（§3-1）。

## 5. エディタの拡張

- **読み込みの自動化**: sspj を検知して自動で ssconverter を呼び、ssab のリソースに変換します（Godot の `EditorImportPlugin` など）。
- **インスペクタ**: インスペクタで、アニメーションのプレビューやテクスチャの差し替えを確かめられるようにします（Godot の `EditorInspectorPlugin` など）。
- **ホットリロード**: デザイナーが SpriteStudio で保存したら、すぐにエディタのアニメーションに反映される仕組みを作れると理想的です。

---
[目次に戻る](../README.ja.md)
