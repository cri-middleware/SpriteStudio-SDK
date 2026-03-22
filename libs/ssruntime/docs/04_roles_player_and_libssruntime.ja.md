# Player と libssruntime の役割分担

libssruntime がサポートする機能と、 Player 側で実装が必要な機能は下記になります。

-  libssruntime 機能
    - ssab データからキーフレームのアニメーションデータの生成
    - アニメーション再生の管理
        - デルタタイムからフレーム番号の計算処理、フレームスキップ(Time-Based Animation) 機能の提供
- Player 機能
    - Player が利用するアセット(テクスチャ、サウンドデータ etc)のロードやメモリ管理
    - アニメーションデータからアニメーションのレンダリング
    - イベント管理
    - ...
