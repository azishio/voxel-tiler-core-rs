extern crate voxel_tiler_core;

use std::fs::File;
use std::io::BufReader;

use gltf::Glb;

use voxel_tiler_core::build_voxelizer::{BuildSimpleVoxelizerDefault, BuildVoxelizer};
use voxel_tiler_core::collection::{PointCloud, VoxelCollection};
use voxel_tiler_core::element::Resolution;
use voxel_tiler_core::glb_gen::GlbGen;
use voxel_tiler_core::mesher::{Mesher, SimpleMesher};
use voxel_tiler_core::ply::PlyStructs;

fn main()
{
    let file = BufReader::new(File::open("examples/data-source/colored_stanford_bunny.ply").unwrap());

    let ply = PlyStructs::from_ply(file);
    let points = ply.into_points();

    let point_cloud = PointCloud::from_points(points);

    let resolution = Resolution::Mater(0.05);

    let voxel_collection = BuildSimpleVoxelizerDefault::voxelize_one(point_cloud, resolution);

    let mesh = SimpleMesher::new(voxel_collection).meshing();

    let glb = Glb::generate(mesh.clone()).unwrap();

    let writer = File::create("examples/colored_stanford_bunny.glb").expect("I/O error");
    glb.to_writer(writer).expect("glTF binary output error");
    println!("Generated colored_stanford_bunny.glb");
}
