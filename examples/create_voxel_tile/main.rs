extern crate voxel_tiler;

use std::fs::File;
use std::io::{BufReader, Write};

use coordinate_transformer::{JprOrigin, ZoomLv};

use voxel_tiler::{PlyStructs, Voxelizer};
use voxel_tiler::default_params::Tile;

fn main() {
    std::fs::create_dir_all("examples/create_voxel/exports").unwrap();

    let file_name = "01JE2421";

    let require_zoom_lv = vec![ZoomLv::Lv17];

    require_zoom_lv.into_iter().for_each(|zoom_lv| {
        let las = BufReader::new(File::open(format!("./examples/data-source/{}.las", file_name)).unwrap());


        let tiled_voxels = Voxelizer::<Tile>::voxelize_from_jpr_las(las, JprOrigin::One, zoom_lv, false);

        tiled_voxels.into_iter().for_each(|(tile_idx, voxel_mesh)| {
            let mut file = File::create(format!("./examples/create_voxel_tile/exports/{}-{}-{}-{}.ply", file_name, tile_idx[0], tile_idx[1], zoom_lv as u32)).expect("Unable to create file");
            println!("{}-{}-{}-{}.ply", file_name, tile_idx[0], tile_idx[1], zoom_lv as u32);

            let ply = PlyStructs::from_voxel_mesh(voxel_mesh).to_ascii_ply_buf();

            file.write_all(&ply).unwrap();
            file.flush().unwrap();
        });
    });
}
