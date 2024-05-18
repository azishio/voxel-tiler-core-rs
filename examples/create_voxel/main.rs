extern crate voxel_tiler;

use std::fs::File;
use std::io::{BufReader, Write};

use coordinate_transformer::{JprOrigin, ZoomLv};

use voxel_tiler::{Offset, PlyStructs, Voxelizer, VoxelizerParam};

struct Param;

impl VoxelizerParam for Param {
    const TILING: bool = false;
    const THRESHOLD: usize = 5;
    const OFFSET: Offset = Offset::Voxel;
}

fn main() {
    std::fs::create_dir_all("examples/create_voxel/exports").unwrap();

    let file_name = "01JE2421";

    let require_zoom_lv = vec![ZoomLv::Lv16, ZoomLv::Lv17];

    require_zoom_lv.into_iter().for_each(|zoom_lv| {
        let las = BufReader::new(File::open(format!("./examples/data-source/{}.las", file_name)).unwrap());


        let v = Voxelizer::<Param>::voxelize_from_jpr_las(las, JprOrigin::One, zoom_lv, false);
        let (_tile_idx, voxel_mesh) = v.into_iter().next().unwrap();

        let mut file = File::create(format!("./examples/create_voxel/exports/{}-{}.ply", file_name, zoom_lv as u32)).expect("Unable to create file");
        println!("{}-{}.ply", file_name, zoom_lv as u32);

        let ply = PlyStructs::from_voxel_mesh(voxel_mesh).to_ascii_ply_buf();

        file.write_all(&ply).unwrap();
        file.flush().unwrap();
    });
}
