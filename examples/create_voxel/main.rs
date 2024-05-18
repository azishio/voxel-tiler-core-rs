extern crate voxel_tiler;

use std::fs::File;
use std::io::{BufReader, Write};

use coordinate_transformer::{JprOrigin, ZoomLv};

use voxel_tiler::{Offset, PlyStructs, Voxelizer, VoxelizerParams};

// Voxelizerのためのパラメータを設定する構造体
// Structure that sets parameters for the Voxelizer
struct Params;

// パラメータの設定
// パラメータについては`voxel_tiler::VoxelizerParam`のドキュメンテーションコメントを参照
// Parameter Settings
// See documentation comments on `voxel_tiler::VoxelizerParam` for parameters
impl VoxelizerParams for Params {
    const TILING: bool = false;
    const THRESHOLD: usize = 5;
    const OFFSET: Offset = Offset::Voxel;
}

fn main() {
    // 出力先のディレクトリを作成
    // Create output destination directory
    std::fs::create_dir_all("examples/create_voxel/exports").unwrap();

    let file_name = "01JE2421";

    // ボクセルデータを生成するズームレベルのリスト
    // List of zoom levels to generate voxel data
    let require_zoom_lv = vec![ZoomLv::Lv16, ZoomLv::Lv17];

    require_zoom_lv.into_iter().for_each(|zoom_lv| {
        // LASファイルを読み込む
        // Load LAS file
        let las = BufReader::new(File::open(format!("./examples/data-source/{}.las", file_name)).unwrap());

        // LASファイルからボクセルデータを生成
        // Generate voxel data from LAS files
        let v = Voxelizer::<Params>::voxelize_from_jpr_las(las, JprOrigin::One, zoom_lv, false);

        // ボクセルデータをPLYファイルに出力
        // VoxelizerParams::TILINGがfalseの場合、Voxelizerが返すVecの要素数は1である
        // output voxel data to PLY file
        // If VoxelizerParams::TILING is false, the number of Vec elements returned by Voxelizer is 1
        let (_tile_idx, voxel_mesh) = v.into_iter().next().unwrap();


        // ボクセルデータをPLYファイルのbufferに変換
        // Convert voxel data to PLY file buffer
        let ply = PlyStructs::from_voxel_mesh(voxel_mesh).to_ascii_ply_buf();

        // ファイルを作成
        // Create file
        let mut file = File::create(format!("./examples/create_voxel/exports/{}-{}.ply", file_name, zoom_lv as u32)).expect("Unable to create file");

        // ファイルに書き込み
        // Write to file
        file.write_all(&ply).unwrap();
        file.flush().unwrap();

        println!("{}-{}.ply", file_name, zoom_lv as u32);
    });
}
