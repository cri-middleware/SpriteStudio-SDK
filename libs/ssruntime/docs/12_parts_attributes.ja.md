# 実装フェーズ 4：パーツ種別とアトリビュートの詳細

SpriteStudio7 には多くのパーツ種別とアトリビュートが存在します。Player 実装の段階に合わせて、対応する項目を増やしていくためのリファレンスです。

## 1. 利用するパーツタイプ

### 基本パーツ
- **PartTypeNull**: 描画を行わないヌルパーツ。親子関係の中継用として使われます。
- **PartTypeNormal**: 通常の矩形パーツ。最も頻繁に使用されます。

### v5〜v6 拡張系パーツ
- **PartTypeInstance**: インスタンスパーツ。別のアニメーションをこのパーツの子として再生します。
- **PartTypeEffect**: エフェクトパーツ。パーティクルエフェクトを表示します。
- **PartTypeMesh**: メッシュパーツ。頂点を変形させて表示します。
- **PartTypeMask**: マスクパーツ。以降の描画にマスクをかけます。

### 7.1 拡張系
- **PartTypeShape**: シェイプパーツ。単純な図形を表示します。
- **PartTypeText**: テキストパーツ。
- **PartTypeNines**: 9スライスパーツ。
- **PartTypeAudio**: 音声再生用のパーツ。
- **PartTypeCamera**: カメラ制御用のパーツ。

### ボーン・関連パーツ
- **PartTypeArmature**, **PartTypeJoint**, **PartTypeMovenode**, **PartTypeConstraint**, **PartTypeBonepoint**, **PartTypeTransformConstraint**

## 2. パーツ操作 API
Player は、データによる指定とは別に、プログラムからパーツの状態を制御する API を提供することを推奨します。

- **setPartVisible(partName, visible)**: 指定したパーツの表示・非表示を強制的に切り替えます。データ上の `AttributeHide` よりも優先して制御したい場合に使用します。

## 3. 利用するアトリビュート

### すべてのパーツ（Null を含む）
- **AttributePosX, AttributePosY, AttributePosZ**: 座標。
- **AttributeRotX, AttributeRotY, AttributeRotZ**: 回転。
- **AttributeSclX, AttributeSclY**: スケール。
- **AttributeLosclX, AttributeLosclY**: ローカルスケール。
- **AttributeAnchorX, AttributeAnchorY**: アンカーポイント（ピボット）。

### 描画系（Normal, Mesh, Nines 等）
- **AttributeCell**: 参照セル。
- **AttributeAlpha, AttributeLoAlpha**: 不透明度。
- **AttributePrio**: 描画優先度（Zオーダー）。
- **AttributeFlipH, AttributeFlipV**: 左右・上下反転。
- **AttributeHide**: 非表示。
- **AttributePivotX, AttributePivotY**: ピボット位置。
- **AttributeSizeX, AttributeSizeY**: サイズ。
- **AttributeImgFlipH, AttributeImgFlipV**: セル（画像）自体の反転。
- **AttributeUvtX, AttributeUvtY, AttributeUvrZ, AttributeUvsX, AttributeUvsY**: UV 座標の移動・回転・スケール。
- **AttributeBoundr**: 当たり判定の半径。

### 特殊系
- **Mask**: `AttributeMask`
- **Effect**: エフェクト開始イベント
- **Instance**: インスタンス再生設定
- **UserData**: ユーザーデータイベント
- **Signal**: シグナルイベント

## 3. この段階での動作確認 (Validation)
主要なアトリビュート（反転、サイズ、ピボット等）が正しく反映されているかを確認します。

- **確認対象**: `Bounding.ssae`
    - **チェック内容**: 
        - 当たり判定用パーツの座標やサイズ（半径）が、正しくアトリビュートとして取得できているか。
        - 反転（Flip）などが親子関係を通じて正しく計算されているか。

---
[目次に戻る](./README.ja.md)
