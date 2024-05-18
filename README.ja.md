[English](README.md) | 日本語

# voxel-tiler-core

![voxel](https://github.com/azishio/voxel-tiler-core-rs/assets/127939746/2c1402c1-03a1-4c05-af64-daa3ea2976a0)

Rustで書かれた高速な点群のボクセル化ライブラリです。
このクレートは、点群データをボクセルメッシュを表現したplyファイルに変換する機能を提供します。
中間処理に関する機能も公開しているため、高度な処理を行いたい場合には部分的に独自の実装を行うこともできます。

## 特徴

+ ピクセル座標に基づいたボクセル化処理
    + このクレートで生成されるボクセルデータの1ボクセルは、ピクセル座標系における1ピクセルに相当します。
    + このため、ボクセルの分解能は緯度とピクセル座標系のズームレベルに依存します。
+ タイルベースの分割
    + 生成するボクセルデータは、オプションで指定されたズームレベルのタイル空間ごとに分割できます。
    + この機能により、大規模な点群データを複数のファイルに分割して出力できます。
+ Las/Lazファイルのインポートに対応
    + Las/Lazファイルから点群データを読み込むことができます。
+ PLYファイルのエクスポートに対忋
    + ボクセルデータを任意の形式(Ascii / Binary)のPLYファイルに出力できます。

## 出力するボクセルデータについて

このクレートが出力するボクセルデータは指定されたピクセル座標系の1ピクセルにフィットするように生成されます。
通常、高さ方向のピクセル座標はありませんが、このクレートでは1ピクセルの辺長を単位長さとして、高さ方向のピクセル座標を定義します。
1ピクセルの辺長は
[coordinate_transformer::pixel_resolution](https://docs.rs/coordinate-transformer/1.5.0/coordinate_transformer/pixel_ll/fn.pixel_resolution.html)
により計算されています。

## 使い方

このクレートには、単純な用途であればすぐに適用できる`VoxelTiler`
構造体が用意されています。この構造体は、Las/Lazファイル若しくは用意された点群データを読み込み、ボクセルメッシュデータを表現するply形式に準拠したバッファーを生成できます。
具体的な使い方は`examples`ディレクトリにあるサンプルコードを参照してください。

## ライセンス

以下のいずれかの下でライセンスされています。

+ Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) または http://www.apache.org/licenses/LICENSE-2.0)
+ MITライセンス([LICENSE-MIT](LICENSE-MIT)または http://opensource.org/licenses/MIT)

(ドキュメンテーションコメント及びREADMEファイルの英語はDeepLにより翻訳されています。)
