English | [日本語](README.ja.md)

# voxel-tiler-core

A crate for voxelizing point clouds and generating 3D tiles.

## Usage

### Basic Usage

```rust
fn example() {
    // Load data
    let file = BufReader::new(File::open("examples/data-source/colored_stanford_bunny.ply").unwrap());

    // Collect vertex information from data
    let point_cloud = PointCloud::from_ply(file);

    // Define resolution
    let resolution = Resolution::Mater(0.03);

    // Voxelize
    let voxel_collection = BuildSimpleVoxelizerDefault::voxelize_one(point_cloud, resolution);

    // Generate mesh
    let mesh = Mesher::meshing(voxel_collection, ValidSide::all());

    // Simplify mesh (optional)
    let simplified_mesh = mesh.simplify();

    // Generate glb file
    let glb = Glb::from_voxel_mesh(mesh.clone()).unwrap();

    // Write file
    let writer = File::create("examples/exports/colored_stanford_bunny.glb").expect("I/O error");
    glb.to_writer(writer).expect("glTF binary output error");
    println!("Generated colored_stanford_bunny.glb");
}
```

### Simple Voxelization

![bunny](https://github.com/user-attachments/assets/9e376fe3-8c39-44f8-8f7a-56e0aaf76a31)

Refer to `examples/generate_voxel_bunny.rs`.

#### How to Run

```shell
cargo run --example bunny --features="ply"
```

### Voxelization and Tiling of Point Clouds

![tile](https://github.com/user-attachments/assets/a17ea91e-47f1-469f-9bfa-c32f2b6c0fe6)

Refer to `examples/generate_voxel_tile.rs`.
When dealing with models with many vertex colors generated from point cloud data, converting them to `glb` files (which
are smaller in size than ASCII ply files) seems to make rendering very costly. The cause is under investigation.

#### How to Run

This takes a very long time. It is recommended to run it in release build.

```shell
cargo run --example tile --features="las ply"
```

### Generating Voxel Terrain Models from Geospatial Information Authority of Japan Elevation Tiles

![terrain](https://github.com/user-attachments/assets/229b83ca-aa93-4942-8a61-8a0681be43d6)

Refer to `generate_terrain_glb`.

#### How to Run

```shell
cargo run --example terrain --features="image"
```

## Supported File Formats

| File Format | Input | Output |
|-------------|-------|--------|
| ply         | ○     | ○      |
| glb         | x     | ○      |
| las         | ○     | x      |
| laz         | ○     | x      |

## License

Licensed under either of

+ Apache License, Version 2.0, ([LICENSE-APACHE](../vec-x-rs/LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
+ MIT license ([LICENSE-MIT](../vec-x-rs/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

(The English in `README.md` and documentation comments was translated from Japanese using ChatGPT and DeepL)
