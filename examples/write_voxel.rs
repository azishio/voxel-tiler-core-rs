extern crate voxel_tiler_core;

use std::fs::File;
use std::io::{BufReader, Write};

use coordinate_transformer::{JprOrigin, ZoomLv};

use voxel_tiler_core::{PlyStructs, Voxelizer};
use voxel_tiler_core::default_params::Fit;

fn main() {
    // 出力先のディレクトリを作成
    // Create output destination directory
    std::fs::create_dir_all("examples/exports").unwrap();

    let file_name = "01JE2423";

    // ボクセルデータを生成するズームレベルのリスト
    // List of zoom levels to generate voxel data
    let require_zoom_lv = vec![ZoomLv::Lv16, ZoomLv::Lv17];

    require_zoom_lv.into_iter().for_each(|zoom_lv| {
        // LASファイルを読み込む
        // Load LAS file
        let las = BufReader::new(File::open(format!("./examples/data-source/{}.las", file_name)).unwrap());

        // LASファイルからボクセルデータを生成
        // Generate voxel data from LAS files
        let v = Voxelizer::<Fit>::voxelize_from_jpr_las(las, JprOrigin::One, zoom_lv, true);

        // VoxelizerParams::TILINGがfalseの場合、Voxelizerが返すVecの要素数は1である
        // If VoxelizerParams::TILING is false, the number of Vec elements returned by Voxelizer is 1
        let (_tile_idx, voxel_mesh) = v.into_iter().next().unwrap();


        // ボクセルデータをPLYファイルのbufferに変換
        // Convert voxel data to PLY file buffer
        let ply = PlyStructs::from_voxel_mesh(voxel_mesh).to_ascii_ply_buf();

        // ファイルを作成
        // Create file
        let mut file = File::create(format!("./examples/exports/{}-{}.ply", file_name, zoom_lv as u32)).expect("Unable to create file");

        // ファイルに書き込み
        // Write to file
        file.write_all(&ply).unwrap();
        file.flush().unwrap();

        println!("{}-{}.ply", file_name, zoom_lv as u32);
    });
}
