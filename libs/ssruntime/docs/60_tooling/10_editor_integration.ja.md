> **日本語** | [English](./10_editor_integration.md)

# エディタインテグレーションとコンバート

プレイヤーを各ゲームエンジン（UE5, Godot, Unity など）に移植する際、ランタイムでの再生だけでなく、「エディタ上で `.sspj` をどのようにインポートし、最適化された `.ssab` に変換するか」というワークフローの構築が重要になります。

本章では、`libssconverter` をエンジンのエディタ拡張に組み込み、**「ディレクトリ単位のバッチコンバート」** と **「プログレスバー連動のキャンセル対応」** を実現するためのベストプラクティスを解説します。この内容は、Godot 版と Unity 版の公式実装で得られた知見に基づいています。

## 1. 全体の設計思想

`libssconverter` の API はバックグラウンドのワーカースレッドで処理を行う非同期設計です。エディタの UI（メインスレッド）をブロックさせないため、以下のアーキテクチャを取ります。

1. **Context の作成と開始**: `ss_converter_create` でコンテキストを作り、処理を開始（`ss_converter_convert` または `ss_converter_scan_dir`）。関数は即座にリターンします。
2. **メインループでのポーリング**: エディタの Update/Process イベント（毎フレーム処理）内で `ss_converter_is_finished` を監視し、進捗ダイアログを更新します。
3. **安全な破棄**: 完了後、結果を受け取って `ss_converter_destroy` でメモリを解放します。

## 2. ディレクトリ単位の再帰的バッチコンバート

ユーザーが複数のフォルダをエディタにドロップした場合など、フォルダ内の `.sspj` を一括で探し出して変換する機能は非常に便利です。`libssconverter` には高速な C-API 走査関数が用意されています。

### 走査のフロー
1. `ss_converter_scan_dir` でバックグラウンド走査を開始します。
2. エディタ側でポーリングしつつ、`ss_converter_scan_found` と `ss_converter_scan_visited` を使って「検索中... n件発見」といった進捗を表示します。
3. 走査が `finished` になったら、`ss_converter_scan_result_count` と `ss_converter_scan_result_at` をループして、見つかった `.sspj` のパスリストを回収します。

### 並列コンバートのスケジュール

見つかった多数の `.sspj` を変換する際、一度にすべて `ss_converter_convert` を発行すると CPU 負荷やメモリ使用量が跳ね上がります。そのため、実行中の Context の数を CPU コア数（例: 最大 4〜8 程度）に制限しながら、空いたスロットに次々と変換ジョブを補充していく（Pump）設計が推奨されます。

## 3. プログレス表示とキャンセル対応

数千のファイルを含む大規模プロジェクトのコンバートや走査において、処理の途中で「キャンセル」ができることは必須の機能です。

### キャンセルの実装ルール

エディタの「キャンセル」ボタンが押された場合、実行中のすべてのコンテキストに対して直ちに **`ss_converter_abort(ctx)`** を呼び出します。これにより、Rust 側のワーカースレッドが安全に中断要求を受け取ります。

> [!WARNING]
> **重要な注意点（UAFの防止）**
> `ss_converter_abort` を呼んだ後も、ワーカースレッドが完全に停止するまでわずかに時間がかかります。そのため、**アボート直後に `ss_converter_destroy` を呼んではいけません**。必ず、引き続きポーリングを行い、**`ss_converter_is_finished(ctx) == true` になってから `ss_converter_destroy(ctx)` を呼び出して**ください。これを守らないと解放済みメモリへのアクセス等でクラッシュする恐れがあります。

### Unity (C#) の実装例

Unity では `EditorApplication.update` を用いてポーリングし、`Progress` API でキャンセルを拾います。

```csharp
// キャンセルコールバック (Progress UIから呼ばれる)
private static bool OnCancel() {
    _canceled = true;
    _pendingJobs.Clear(); // 待機キューをクリア
    unsafe {
        foreach (var ctxPtr in _activeContexts) {
            var ctx = (SpriteStudio.SsConverter.Context*)ctxPtr;
            // 実行中のコンテキストに中断命令を出す
            SpriteStudio.SsConverter.NativeMethods.ss_converter_abort(ctx);
        }
    }
    return true; // ポーリングループ側で安全に回収させるため、ここでは Destroy しない
}

// Update ポーリングループ
private static void Poll() {
    for (int i = _activeContexts.Count - 1; i >= 0; i--) {
        unsafe {
            var ctx = (SpriteStudio.SsConverter.Context*)_activeContexts[i];
            
            // 完了するまで待つ (Abort発行後もここでFinishedになるのを待機)
            if (!SpriteStudio.SsConverter.NativeMethods.ss_converter_is_finished(ctx)) continue;
            
            try {
                // 結果の処理 (ソースマップへの登録など)
            } finally {
                // 例外が起きても必ず finished になってから destroy する
                SpriteStudio.SsConverter.NativeMethods.ss_converter_destroy(ctx);
                _activeContexts.RemoveAt(i);
            }
        }
    }
}
```

### Godot (C++) の実装例

Godot の場合は `NOTIFICATION_PROCESS`（毎フレームの Update）を用いて、カスタムの ProgressDialog と連携します。

```cpp
void SSImporter::_notification(int p_what) {
    if (p_what != NOTIFICATION_PROCESS) return;

    if (_is_converting) {
        // プログレスダイアログの Cancel が押されたか検知
        if (!_convert_canceling && _import_dialog->is_canceled()) {
            _convert_canceling = true;
            for (int i = 0; i < _active_ctx.size(); i++) {
                ss_converter_abort((Context *)_active_ctx[i]); // 中断要求
            }
        }

        // アクティブな変換のポーリング
        for (int i = _active_ctx.size() - 1; i >= 0; i--) {
            Context *ctx = (Context *)_active_ctx[i];
            
            // 中断要求後も、完全に終了するまで待つ
            if (!ss_converter_is_finished(ctx)) continue;

            // アボートされていなければ結果を処理
            CConverterError result = ss_converter_get_result(ctx);
            if (result == CConverterError::Success) {
                // ... 成功時の処理 ...
            }
            
            // スレッドが終了したことが保証されたので安全に破棄
            ss_converter_destroy(ctx);
            _active_ctx.remove_at(i);
        }
    }
}
```

## 4. エラーハンドリングと自己回復

エディタ統合において、例外やエラーでコンバータのポーリングループが停止してしまうと、バックグラウンドのワーカースレッドや C のメモリ（Context）が永久にリークしてしまいます。

- **C# 等での例外処理**: 変換完了後の C# 側の結果処理（ファイル列挙など）で例外（Exception）が投げられても、`try / finally` ブロックを用いて必ず `ss_converter_destroy` を呼び出し、ポーリングループが止まらないように自己回復させる設計が重要です。

## 5. エディタ拡張とワークフローの統合

開発効率向上のため、ゲームエンジンのエディタ拡張として `ssconverter` を統合することを強く推奨します。

- **Godot (C++) の例**:
    - `EditorImportPlugin`: `.sspj` を検知して自動的に `ssconverter` を呼び出し、内部的な `.ssab` リソースに変換します。
    - `EditorInspectorPlugin`: インスペクタ上でアニメーションのプレビューや、テクスチャオーバーライドの確認ができるようにします。
- **ワークフロー**: デザイナーが SpriteStudio で保存した瞬間に、ゲームエディタ側で最新のアニメーションが即座に反映される環境（ホットリロード）を構築することが理想です。

---
[Player移植ガイド Top](../README.ja.md)
