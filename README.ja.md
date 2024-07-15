[English](README.md) | 日本語

# voxel-tiler-core

点群をボクセル化し、3Dタイルを生成するためのクレートです。

## 特徴

+ ピクセル座標に基づいたボクセル化処理
    + このクレートで生成されるボクセルデータの1ボクセルは、ピクセル座標系における1ピクセルに相当します。
    + これにより、複数のモデル間での位置的な整合性が担保されます。
+ タイルベースの分割
    + 生成するボクセルデータは、オプションで指定されたズームレベルのタイル空間ごとに分割できます。
    + この機能により、大規模な点群データを複数のファイルに分割して出力できます。

## 対応ファイル形式

| ファイル形式 | 入力 | 出力 |
|--------|----|----|
| ply    | ○  | ○  |
| glb    | x  | ○  |
| las    | ○  | x  |
| laz    | ○  | x  |

## 使い方

### 基本的な使い方

```rust 
fn example() {
    // データの読み込み
    let file = BufReader::new(File::open("examples/data-source/colored_stanford_bunny.ply").unwrap());

    // データから頂点情報を収集
    let point_cloud = PointCloud::from_ply(file);

    // 分解能の定義
    let resolution = Resolution::Mater(0.03);

    // ボクセル化
    let voxel_collection = BuildSimpleVoxelizerDefault::voxelize_one(point_cloud, resolution);

    // メッシュの生成
    let mesh = Mesher::meshing(voxel_collection, ValidSide::all());

    // メッシュの簡略化（任意）
    let simplified_mesh = mesh.simplify();

    // glbファイルの生成
    let glb = Glb::from_voxel_mesh(mesh.clone()).unwrap();

    // ファイルの書き込み
    let writer = File::create("examples/exports/colored_stanford_bunny.glb").expect("I/O error");
    glb.to_writer(writer).expect("glTF binary output error");
    println!("Generated colored_stanford_bunny.glb");
}
```

### 単純なボクセル化

![bunny](https://github.com/user-attachments/assets/9e376fe3-8c39-44f8-8f7a-56e0aaf76a31)

`examples/generate_voxel_bunny.rs`を参照してください。

#### 実行方法

```shell
cargo run --example bunny --features="ply"
```

### 点群のボクセル化とタイル分割

![tile](https://github.com/user-attachments/assets/a17ea91e-47f1-469f-9bfa-c32f2b6c0fe6)

`examples/generate_voxel_tile.rs`を参照してください。
点群データから生成したような多数の頂点カラーを持つモデルを扱う場合、`glb`ファイルにすると(ASCII
plyよりもファイルサイズは小さいですが)レンダリングが非常に高コストになるようです。
原因は調査中です。

#### 実行方法

これは非常に時間がかかります。リリースビルドにして実行することをお勧めします。

```shell
cargo run --example tile --features="las ply"
```

### 国土地理院の標高タイルからボクセル地形モデルの生成

![terrain](https://github.com/user-attachments/assets/229b83ca-aa93-4942-8a61-8a0681be43d6)

`generate_terrain_glb`を参照してください。

#### 実行方法

```shell
cargo run --example terrain --features="image"
```

## ライセンス

Licensed under either of

+ Apache License, Version 2.0, ([LICENSE-APACHE](../vec-x-rs/LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
+ MIT license ([LICENSE-MIT](../vec-x-rs/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

(`README.md`とドキュメンテーションコメントの英語はChatGPTとDeepLによって日本語から英訳されました)
