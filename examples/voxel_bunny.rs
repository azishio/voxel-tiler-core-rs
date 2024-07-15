extern crate voxel_tiler_core;

use std::fs::File;
use std::io::{BufReader, Write};

use gltf::Glb;

use voxel_tiler_core::build_voxelizer::{BuildSimpleVoxelizerDefault, BuildVoxelizer};
use voxel_tiler_core::collection::PointCloud;
use voxel_tiler_core::glb::{ColorMode, GlbGen};
use voxel_tiler_core::mesh::{Mesher, ValidSide};
use voxel_tiler_core::ply::PlyStructs;
use voxel_tiler_core::voxelizer::Resolution;

fn main()
{
    let file = BufReader::new(File::open("examples/data-source/colored_stanford_bunny.ply").unwrap());

    let point_cloud = PointCloud::from_ply(file);

    let resolution = Resolution::Mater(0.03);

    let voxel_collection = BuildSimpleVoxelizerDefault::voxelize_one(point_cloud, resolution);

    let mesh = Mesher::meshing(voxel_collection, ValidSide::all()).simplify();

    {
        let glb = Glb::from_voxel_mesh(mesh.clone(), ColorMode::Srgb).unwrap();

        let writer = File::create("examples/exports/bunny.glb").expect("I/O error");
        glb.to_writer(writer).expect("glTF binary output error");
        println!("Generated bunny.glb");
    }


    {
        let ply = PlyStructs::from_voxel_mesh(mesh);

        let buf = ply.into_ascii_buf();

        let mut writer = File::create("examples/exports/bunny.ply").expect("I/O error");
        writer.write_all(&buf).expect("I/O error");
        writer.flush().expect("I/O error");
        println!("Generated bunny.ply");
    }
}
