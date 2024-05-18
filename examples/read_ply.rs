extern crate voxel_tiler_core;

use std::fs::File;

use voxel_tiler_core::PlyStructs;

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
    let ply_by_buf = PlyStructs::from_ply(file.as_slice());
    println!("{:?}", ply_by_buf);

    // 対応していないプロパティは警告をプリントして無視されます。
    // 警告をプリントさせたくない場合は、`print-warning`featureフラグをおろしてください。
    // Unsupported properties will print a warning and be ignored.
    // If you do not want warnings to be printed, please turn off the `print-warning` feature flag.
    let file = File::open("examples/data-source/have_unsupported_properties_box.ply").unwrap();
    let ply = PlyStructs::from_ply(file);
    println!("{:?}", ply);

    // `box.ply`と`have_unsupported_properties_box.ply`において、対応するプロパティは同じです。
    // In `box.ply` and `have_unsupported_properties_box.ply`, the corresponding properties are the same.
    assert_eq!(ascii_ply.to_ascii_ply_buf(), ply.to_ascii_ply_buf());

    // red, green, blueのうち、存在しないプロパティは0で埋められます。
    let file = File::open("examples/data-source/cone.ply").unwrap();
    let ply = PlyStructs::from_ply(file);
    println!("{:?}", ply);
}
