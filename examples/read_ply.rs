extern crate voxel_tiler;

use std::fs::File;

use voxel_tiler::PlyStructs;

fn main() {
    // Ascii PLY
    let file = File::open("examples/data-source/box.ply").unwrap();
    let ascii_ply = PlyStructs::from_ply(file);
    println!("{:?}", ascii_ply);

    // Binary PLY
    let file = File::open("examples/data-source/binary_box.ply").unwrap();
    let binary_ply = PlyStructs::from_ply(file);
    println!("{:?}", binary_ply);

    // from buffer
    let file: Vec<u8> = std::fs::read("examples/data-source/box.ply").unwrap();
    let ply = PlyStructs::from_ply(file.as_slice());
    println!("{:?}", ply);
}
